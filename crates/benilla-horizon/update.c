// wow-nx: the self-updater, after haloce-nx's (port/switch/host/host_update.c, by thelinkin3000 and nx-mod, CC0).
//
// A release build knows its build number (WOWNX_BUILD_NUMBER, given by switch/release.sh), which names its GitHub
// release: build-<number>. At start a thread asks GitHub where nx-mod/wow-nx's /releases/latest redirects to
// (/releases/tag/build-<number>): one request, no JSON, while the game loads, so the start waits for nothing. When the
// deko3d backend takes the window (screen.c), a newer build found by then is offered on libnx's text console; taken, the release's wow-nx-switch.zip downloads, and since the archive is stored (zip -0)
// with each file's size and CRC ahead of its data, wow-nx.nro is unpacked as it arrives, beside the running program
// as wow-nx.nro.new, checked against its CRC, and only then put in place. The archive itself is never kept.
//
// The loader read the program whole when it started, so the running one is replaced at once, and the player starts
// the game again to run the new one. Builds without a number never look; WOWNX_UPDATE=0 in benilla.env turns the
// look off.
#include <stdatomic.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <switch.h>

#include "update.h"

#ifndef WOWNX_BUILD_NUMBER
#define WOWNX_BUILD_NUMBER 0
#endif

#define UPDATE_REPOSITORY "nx-mod/wow-nx"
#define UPDATE_ASSET "wow-nx-switch.zip"
#define UPDATE_PROGRAM "wow-nx.nro"
// Where the program is when the loader does not say.
#define DEFAULT_PROGRAM "sdmc:/wow-nx/" UPDATE_PROGRAM

enum {
    ZIP_LOCAL_HEADER = 30,
    ZIP_LOCAL_SIGNATURE = 0x04034b50,
    ZIP_CENTRAL_SIGNATURE = 0x02014b50,
    PROGRESS_EVERY = 1024 * 1024,
};

static FILE *log_file;
static PadState pad;
// The latest release's build number, once the check has it (0: none could be learned).
static atomic_long latest = -1;
static Thread check;

void host_logf(int priority, const char *format, ...) {
    if (!log_file && !(log_file = fopen("sdmc:/wow-nx/update.txt", "w"))) return;
    va_list arguments;
    va_start(arguments, format);
    fprintf(log_file, "%c ", priority >= HOST_LOG_ERROR ? 'E' : priority >= HOST_LOG_WARN ? 'W' : 'I');
    vfprintf(log_file, format, arguments);
    fputc('\n', log_file);
    va_end(arguments);
    fflush(log_file);
}

static int fail(char *error, size_t error_size, const char *format, ...) __attribute__((format(printf, 3, 4)));
static int fail(char *error, size_t error_size, const char *format, ...) {
    va_list arguments;
    va_start(arguments, format);
    vsnprintf(error, error_size, format, arguments);
    va_end(arguments);
    host_logf(HOST_LOG_WARN, "update: %s", error);
    return 0;
}

// The program's path: the loader's first argument, or the usual place.
static void program_path(char *path, size_t size) {
    const char *arguments = envGetArgv();
    size_t length = 0;
    snprintf(path, size, "%s", DEFAULT_PROGRAM);
    if (!arguments) return;
    if (*arguments == '"') {
        const char *end = strchr(arguments + 1, '"');
        if (!end) return;
        length = (size_t)(end - arguments - 1);
        arguments++;
    } else {
        while (arguments[length] && arguments[length] != ' ') length++;
    }
    if (length < 5 || length >= size || strncmp(arguments + length - 4, ".nro", 4) || strncmp(arguments, "sdmc:/", 6))
        return;
    memcpy(path, arguments, length);
    path[length] = 0;
}

// The latest release's build number, 0 if it cannot be learned.
static long latest_build(void) {
    char location[HOST_HTTPS_MAXIMUM_URL];
    char error[256];
    int status = host_https_get("https://github.com/" UPDATE_REPOSITORY "/releases/latest", 0, NULL, NULL, location,
                                sizeof(location), error, sizeof(error));
    if (status < 300 || status >= 400) {
        if (status >= 0) host_logf(HOST_LOG_INFO, "update: GitHub answered %d for the latest release", status);
        return 0;
    }
    const char *tag = strstr(location, "/releases/tag/build-");
    return tag ? strtol(tag + strlen("/releases/tag/build-"), NULL, 10) : 0;
}

// ---------- unpacking as it downloads

static unsigned long crc_table[256];

static unsigned long crc_update(unsigned long crc, const unsigned char *data, size_t size) {
    if (!crc_table[1]) {
        for (unsigned long entry = 0; entry < 256; entry++) {
            unsigned long value = entry;
            for (int bit = 0; bit < 8; bit++) value = value & 1 ? 0xedb88320UL ^ (value >> 1) : value >> 1;
            crc_table[entry] = value;
        }
    }
    crc = ~crc & 0xffffffffUL;
    for (size_t index = 0; index < size; index++) crc = crc_table[(crc ^ data[index]) & 0xff] ^ (crc >> 8);
    return ~crc & 0xffffffffUL;
}

static unsigned long little(const unsigned char *bytes, int count) {
    unsigned long value = 0;
    for (int index = count - 1; index >= 0; index--) value = (value << 8) | bytes[index];
    return value;
}

enum { UNPACK_HEADER, UNPACK_NAME, UNPACK_DATA, UNPACK_DONE };

struct unpack {
    int state;
    unsigned char header[ZIP_LOCAL_HEADER];
    size_t have;
    char name[256];
    size_t name_length, extra_length;
    unsigned long remaining, expected_crc, crc;
    // The program being written, if this entry is it.
    FILE *file;
    char new_path[520];
    int written;
    long long received, reported, total;
    long build;
    char error[256];
};

static void draw_progress(const struct unpack *unpack) {
    double done = (double)unpack->received / (1024 * 1024);
    if (unpack->total > 0)
        printf("\r  Build %ld: %.1f of %.1f MB   ", unpack->build, done, (double)unpack->total / (1024 * 1024));
    else
        printf("\r  Build %ld: %.1f MB   ", unpack->build, done);
    consoleUpdate(NULL);
}

static int unpack_failed(struct unpack *unpack, const char *format, const char *detail) {
    snprintf(unpack->error, sizeof(unpack->error), format, detail);
    return 0;
}

// Starts the entry whose header and name have arrived.
static int unpack_entry(struct unpack *unpack) {
    const unsigned char *header = unpack->header;
    unsigned long flags = little(header + 6, 2), method = little(header + 8, 2);
    unsigned long compressed = little(header + 18, 4), size = little(header + 22, 4);
    unpack->name[unpack->name_length < sizeof(unpack->name) ? unpack->name_length : sizeof(unpack->name) - 1] = 0;
    unpack->remaining = compressed;
    unpack->expected_crc = little(header + 14, 4);
    unpack->crc = 0;
    unpack->file = NULL;
    if (strcmp(unpack->name, UPDATE_PROGRAM)) return 1;
    // The release is stored and says each size up front (switch/release.sh); anything else is an archive this was
    // not written for.
    if (method != 0 || (flags & 8) || compressed != size)
        return unpack_failed(unpack, "The release's %s is packed in a way the console cannot unpack.", unpack->name);
    unpack->file = fopen(unpack->new_path, "wb");
    if (!unpack->file) return unpack_failed(unpack, "Cannot write %s to the card.", unpack->new_path);
    setvbuf(unpack->file, NULL, _IOFBF, 256 * 1024);
    return 1;
}

static int unpack_finish_entry(struct unpack *unpack) {
    if (!unpack->file) return 1;
    int closed = fclose(unpack->file) == 0;
    unpack->file = NULL;
    if (!closed) return unpack_failed(unpack, "Writing %s to the card failed.", unpack->name);
    if (unpack->crc != unpack->expected_crc) return unpack_failed(unpack, "%s did not arrive intact.", unpack->name);
    unpack->written = 1;
    return 1;
}

static int unpack_body(void *context, const void *data, size_t size, long long total) {
    struct unpack *unpack = context;
    const unsigned char *bytes = data;
    unpack->total = total;
    unpack->received += (long long)size;
    if (unpack->received - unpack->reported >= PROGRESS_EVERY) {
        unpack->reported = unpack->received;
        draw_progress(unpack);
        padUpdate(&pad);
        if (padGetButtonsDown(&pad) & HidNpadButton_B) return unpack_failed(unpack, "%s", "The update was stopped.");
    }
    while (size && unpack->state != UNPACK_DONE) {
        size_t take = 0;
        switch (unpack->state) {
        case UNPACK_HEADER:
            take = ZIP_LOCAL_HEADER - unpack->have < size ? ZIP_LOCAL_HEADER - unpack->have : size;
            memcpy(unpack->header + unpack->have, bytes, take);
            unpack->have += take;
            if (unpack->have >= 4 && little(unpack->header, 4) == ZIP_CENTRAL_SIGNATURE) {
                // Past the files: the directory at the end is not needed.
                unpack->state = UNPACK_DONE;
                break;
            }
            if (unpack->have == ZIP_LOCAL_HEADER) {
                if (little(unpack->header, 4) != ZIP_LOCAL_SIGNATURE)
                    return unpack_failed(unpack, "%s", "The release is not an archive the console can read.");
                unpack->name_length = little(unpack->header + 26, 2);
                unpack->extra_length = little(unpack->header + 28, 2);
                unpack->have = 0;
                unpack->state = UNPACK_NAME;
            }
            break;
        case UNPACK_NAME:
            take = unpack->name_length + unpack->extra_length - unpack->have;
            take = take < size ? take : size;
            // The name, then the extra field, which is passed over.
            for (size_t index = 0; index < take; index++) {
                size_t at = unpack->have + index;
                if (at < unpack->name_length && at < sizeof(unpack->name) - 1) unpack->name[at] = (char)bytes[index];
            }
            unpack->have += take;
            if (unpack->have == unpack->name_length + unpack->extra_length) {
                unpack->have = 0;
                if (!unpack_entry(unpack)) return 0;
                unpack->state = UNPACK_DATA;
                if (!unpack->remaining) {
                    if (!unpack_finish_entry(unpack)) return 0;
                    unpack->state = UNPACK_HEADER;
                }
            }
            break;
        case UNPACK_DATA:
            take = unpack->remaining < size ? unpack->remaining : size;
            if (unpack->file) {
                if (fwrite(bytes, 1, take, unpack->file) != take)
                    return unpack_failed(unpack, "Writing %s to the card failed.", unpack->name);
                unpack->crc = crc_update(unpack->crc, bytes, take);
            }
            unpack->remaining -= take;
            if (!unpack->remaining) {
                if (!unpack_finish_entry(unpack)) return 0;
                unpack->state = UNPACK_HEADER;
            }
            break;
        }
        bytes += take;
        size -= take;
    }
    return 1;
}

// ---------- the console's part

static u64 wait_for(u64 buttons) {
    for (;;) {
        if (!appletMainLoop()) {
            consoleExit(NULL);
            exit(0);
        }
        padUpdate(&pad);
        u64 down = padGetButtonsDown(&pad);
        if (down & buttons) return down;
        svcSleepThread(16666667);
    }
}

// Downloads build's archive and puts its program in place of the running one's file. 1 when installed.
static int download_and_install(long build, char *error, size_t error_size) {
    char url[256], target[512];
    struct unpack *unpack = calloc(1, sizeof(*unpack));
    if (!unpack) return fail(error, error_size, "Out of memory for the update.");
    program_path(target, sizeof(target));
    snprintf(unpack->new_path, sizeof(unpack->new_path), "%s.new", target);
    snprintf(url, sizeof(url), "https://github.com/" UPDATE_REPOSITORY "/releases/download/build-%ld/" UPDATE_ASSET,
             build);
    host_logf(HOST_LOG_INFO, "update: downloading %s to %s", url, unpack->new_path);
    unpack->build = build;
    draw_progress(unpack);
    int status = host_https_get(url, 1, unpack_body, unpack, NULL, 0, error, error_size);
    if (unpack->file) fclose(unpack->file);
    int written = unpack->written;
    if (unpack->error[0]) {
        fail(error, error_size, "%s", unpack->error);
        status = -1;
    } else if (status >= 0 && status != 200) {
        fail(error, error_size, "GitHub answered %d for build %ld's %s.", status, build, UPDATE_ASSET);
        status = -1;
    } else if (status == 200 && !written) {
        fail(error, error_size, "Build %ld's archive has no %s.", build, UPDATE_PROGRAM);
        status = -1;
    }
    if (status != 200) {
        remove(unpack->new_path);
        free(unpack);
        return 0;
    }
    // The card will not rename onto a file that is there: the old one goes first.
    remove(target);
    if (rename(unpack->new_path, target) != 0) {
        fail(error, error_size, "Cannot put %s in place; it is on the card as %s.", target, unpack->new_path);
        free(unpack);
        return 0;
    }
    free(unpack);
    return 1;
}

static void check_latest(void *unused) {
    (void)unused;
    atomic_store(&latest, latest_build());
}

// With the network up: starts the look for a newer release, on its own thread.
void wownx_update_start(void) {
    if (WOWNX_BUILD_NUMBER <= 0) return;
    const char *setting = getenv("WOWNX_UPDATE");
    if (setting && !strcmp(setting, "0")) {
        host_logf(HOST_LOG_INFO, "update: WOWNX_UPDATE=0 in benilla.env");
        return;
    }
    // TLS and the zip's buffers want room: 256 KiB of stack, below the main thread's priority.
    if (R_FAILED(threadCreate(&check, check_latest, NULL, NULL, 256 * 1024, 0x2D, -2)) || R_FAILED(threadStart(&check)))
        host_logf(HOST_LOG_WARN, "update: the check could not start");
}

// When the deko3d backend takes the window (screen.c), the loading picture just closed: a newer release found by now
// is offered on the console, and installed if the player agrees, after which the game quits to be started again. A
// check still asking waits for the next start.
void wownx_update_offer(void) {
    long build = atomic_load(&latest);
    if (build < 0) {
        if (WOWNX_BUILD_NUMBER > 0 && check.handle) host_logf(HOST_LOG_INFO, "update: the check had no answer yet");
        return;
    }
    if (build <= WOWNX_BUILD_NUMBER) {
        host_logf(HOST_LOG_INFO, "update: build %d is the latest%s", WOWNX_BUILD_NUMBER,
                  build ? "" : " that could be checked");
        return;
    }
    host_logf(HOST_LOG_INFO, "update: build %ld is available (this is build %d)", build, WOWNX_BUILD_NUMBER);
    padConfigureInput(1, HidNpadStyleSet_NpadStandard);
    padInitializeDefault(&pad);
    consoleInit(NULL);
    printf("\n  wow-nx\n\n  Build %ld is available; this is build %d.\n\n  A  Update now\n  B  Not now\n", build,
           WOWNX_BUILD_NUMBER);
    consoleUpdate(NULL);
    if (wait_for(HidNpadButton_A | HidNpadButton_B) & HidNpadButton_A) {
        char error[256];
        printf("\n  Downloading (B stops)...\n\n");
        consoleUpdate(NULL);
        if (download_and_install(build, error, sizeof(error))) {
            host_logf(HOST_LOG_INFO, "update: build %ld is installed", build);
            printf("\n\n  Build %ld is installed. Start the game again to run it.\n\n  A  Quit\n", build);
            consoleUpdate(NULL);
            wait_for(HidNpadButton_A);
            consoleExit(NULL);
            exit(0);
        }
        printf("\n\n  The update did not finish:\n  %s\n\n  B  Continue without it\n", error);
        consoleUpdate(NULL);
        wait_for(HidNpadButton_B);
    } else {
        host_logf(HOST_LOG_INFO, "update: not now");
    }
    // The window goes back to the deko3d backend, which shows the loading picture again.
    consoleExit(NULL);
}
