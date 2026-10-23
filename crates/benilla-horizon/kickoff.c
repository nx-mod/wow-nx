// wow-nx: what a failed GPU kickoff was. deko3d reports only "gpu channel kickoff failed" (dk_queue.cpp) and drops
// libnx's result; a binary linked with -Wl,--wrap=nvGpuChannelKickoff (benilla-app's build.rs) comes through here,
// and the first failure writes the result, the rejected GPFIFO entries and the kickoffs before it to
// sdmc:/wow-nx/kickoff.txt. libnx already retries a kickoff the kernel answers Busy (a full ring), so whatever
// lands here the kernel refused outright. Two kickoffs at once (the queue used from two threads) are written too.
#include <stdatomic.h>
#include <stdio.h>
#include <switch.h>

// Weak, so a binary that does not wrap the symbol still links (this archive is whole-archive).
extern Result __real_nvGpuChannelKickoff(NvGpuChannel* c) __attribute__((weak));

typedef struct {
    u64 thread;
    u64 tick;
    u32 entries;
    u64 first, last;
} Kickoff;

enum { HISTORY = 16 };
static Kickoff history[HISTORY];
static u32 kickoffs;
static atomic_int inside;
static atomic_bool reported_failure, reported_overlap;

static u64 current_thread(void) {
    u64 id = 0;
    svcGetThreadId(&id, CUR_THREAD_HANDLE);
    return id;
}

// An entry's GPU address (bits 2..39) and length in words (bits 10..30 of its second word).
static void write_entry(FILE* f, const char* what, u64 desc) {
    u64 address = desc & 0xfffffffffcull;
    u32 words = (u32)(desc >> 42) & 0x1fffff;
    fprintf(f, "%s %016llx: address %010llx, %u words%s\n", what, (unsigned long long)desc, (unsigned long long)address,
        words, (desc >> 41) & 1 ? ", not main" : "");
}

static void report(NvGpuChannel* c, Result rc, u64 thread) {
    FILE* f = fopen("sdmc:/wow-nx/kickoff.txt", "w");
    if (!f) return;
    fprintf(f, "kickoff %u failed: result 0x%x (module %u, description %u), thread %llu, %u entries, fence incr %u\n",
        kickoffs, rc, R_MODULE(rc), R_DESCRIPTION(rc), (unsigned long long)thread, c->num_entries, c->fence_incr);
    NvError error = {0};
    if (R_SUCCEEDED(nvGpuChannelGetErrorInfo(c, &error)))
        fprintf(f, "channel error info: type %u, info %u %u %u %u\n", error.type, error.info[0], error.info[1],
            error.info[2], error.info[3]);
    for (u32 i = 0; i < c->num_entries && i < GPFIFO_QUEUE_SIZE; i++) {
        char what[24];
        snprintf(what, sizeof(what), "entry %u", i);
        write_entry(f, what, c->entries[i].desc);
    }
    fprintf(f, "the kickoffs before it, oldest first:\n");
    u32 n = kickoffs < HISTORY ? kickoffs : HISTORY;
    for (u32 i = kickoffs - n; i < kickoffs; i++) {
        const Kickoff* k = &history[i % HISTORY];
        fprintf(f, "#%u thread %llu tick %llu, %u entries\n", i, (unsigned long long)k->thread,
            (unsigned long long)k->tick, k->entries);
        write_entry(f, "  first", k->first);
        write_entry(f, "  last ", k->last);
    }
    fclose(f);
}

Result __wrap_nvGpuChannelKickoff(NvGpuChannel* c) {
    u64 thread = current_thread();
    if (atomic_fetch_add(&inside, 1) != 0 && !atomic_exchange(&reported_overlap, true)) {
        FILE* f = fopen("sdmc:/wow-nx/kickoff-overlap.txt", "w");
        if (f) {
            fprintf(f, "two kickoffs at once: thread %llu entered during another (kickoff %u)\n",
                (unsigned long long)thread, kickoffs);
            fclose(f);
        }
    }
    Kickoff k = {thread, armGetSystemTick(), c->num_entries, 0, 0};
    if (c->num_entries) {
        k.first = c->entries[0].desc;
        k.last = c->entries[c->num_entries - 1].desc;
    }
    Result rc = __real_nvGpuChannelKickoff(c);
    if (R_FAILED(rc) && !atomic_exchange(&reported_failure, true))
        report(c, rc, thread);
    else if (R_SUCCEEDED(rc) && k.entries)
        history[kickoffs++ % HISTORY] = k;
    atomic_fetch_sub(&inside, 1);
    return rc;
}
