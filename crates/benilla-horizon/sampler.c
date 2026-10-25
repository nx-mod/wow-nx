// wow-nx: a sampling profiler, on with WOWNX_PROFILE=1 in benilla.env. A thread pauses each of the game's threads 250
// times a second (svcSetThreadActivity), reads where it is (svcGetThreadContext3: its PC and its caller's, LR), and
// lets it go; every 30 seconds the window's counts go to sdmc:/wow-nx/samples.txt, as offsets into the program, which
// switch/profile.py names from the ELF. A thread waiting shows as the wait it is in.
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <switch.h>

enum {
    MAX_THREADS = 64,
    // (thread, address, PC or LR) counts: open addressing, a power of two.
    SLOTS = 1 << 16,
    PERIOD_NS = 4000000,
    WINDOW_NS = 30000000000ULL,
};

struct slot {
    u64 key;
    u32 count;
};

static Handle threads[MAX_THREADS];
static atomic_int thread_count;
static struct slot *table;
static u64 text_base;
static u32 samples[MAX_THREADS];
static u32 lost;

// thread.c: each new thread, as it starts.
void wownx_sampler_thread(Handle handle) {
    int at = atomic_fetch_add(&thread_count, 1);
    if (at < MAX_THREADS) threads[at] = handle;
}

// The key: thread in the top 7 bits, a PC/LR bit, the address's offset into the program below.
static void count(int thread, int is_lr, u64 address) {
    if (address < text_base) return;
    u64 key = ((u64)thread << 57) | ((u64)is_lr << 56) | ((address - text_base) & ((1ULL << 56) - 1));
    u64 at = (key * 0x9e3779b97f4a7c15ULL) >> 48;
    for (int probe = 0; probe < 64; probe++, at = (at + 1) & (SLOTS - 1)) {
        if (table[at].key == key) {
            table[at].count++;
            return;
        }
        if (!table[at].count) {
            table[at].key = key;
            table[at].count = 1;
            return;
        }
    }
    lost++;
}

static void dump(int window) {
    FILE *f = fopen("sdmc:/wow-nx/samples.txt", "w");
    if (f) {
        int n = atomic_load(&thread_count);
        fprintf(f, "# wow-nx samples, window %d (%d s), %d threads, every %d us, %u lost\n", window,
                (int)(WINDOW_NS / 1000000000ULL), (int)(n < MAX_THREADS ? n : MAX_THREADS), (int)(PERIOD_NS / 1000), lost);
        for (int t = 0; t < n && t < MAX_THREADS; t++) fprintf(f, "thread %d %u\n", t, samples[t]);
        for (int at = 0; at < SLOTS; at++) {
            if (!table[at].count) continue;
            u64 key = table[at].key;
            fprintf(f, "%d %c %llx %u\n", (int)(key >> 57), (key >> 56) & 1 ? 'L' : 'P',
                    (unsigned long long)(key & ((1ULL << 56) - 1)), table[at].count);
        }
        fclose(f);
    }
    memset(table, 0, SLOTS * sizeof(*table));
    memset(samples, 0, sizeof(samples));
    lost = 0;
}

static void sample(void *unused) {
    (void)unused;
    u64 window_start = armGetSystemTick();
    int window = 0;
    for (;;) {
        svcSleepThread(PERIOD_NS);
        int n = atomic_load(&thread_count);
        for (int t = 0; t < n && t < MAX_THREADS; t++) {
            Handle h = threads[t];
            if (R_FAILED(svcSetThreadActivity(h, ThreadActivity_Paused))) continue;
            ThreadContext context;
            Result rc = svcGetThreadContext3(&context, h);
            svcSetThreadActivity(h, ThreadActivity_Runnable);
            if (R_FAILED(rc)) continue;
            samples[t]++;
            count(t, 0, context.pc.x);
            count(t, 1, context.lr);
        }
        if (armTicksToNs(armGetSystemTick() - window_start) >= WINDOW_NS) {
            dump(++window);
            window_start = armGetSystemTick();
        }
    }
}

void wownx_sampler_start(void) {
    const char *setting = getenv("WOWNX_PROFILE");
    if (!setting || strcmp(setting, "1")) return;
    MemoryInfo info;
    u32 page;
    if (R_FAILED(svcQueryMemory(&info, &page, (u64)&wownx_sampler_start))) return;
    text_base = info.addr;
    table = calloc(SLOTS, sizeof(*table));
    if (!table) return;
    // The main thread is thread 0; thread.c adds the rest as they start.
    wownx_sampler_thread(envGetMainThreadHandle());
    static Thread thread;
    if (R_SUCCEEDED(threadCreate(&thread, sample, NULL, NULL, 64 * 1024, 0x2B, -2))) threadStart(&thread);
}
