// wow-nx: threads across the cores. libnx's pthread_create (newlib.c, __syscall_thread_create) makes every thread
// with the process's ideal core, core 0, and widens it to the process's core mask; Horizon moves a thread off its
// ideal core only when another core sits idle, so Bevy's workers, the render thread, audio and IO all queued behind
// each other on core 0 first. Each new thread takes the next core as its ideal one, keeping the whole mask.
#include <stdatomic.h>
#include <switch.h>

struct __pthread_t;
int __real___syscall_thread_create(struct __pthread_t** thread, void* (*func)(void*), void* arg, void* stack_addr,
                                   size_t stack_size);

static u64 core_mask(void) {
    u64 mask = 0;
    if (R_FAILED(svcGetInfo(&mask, InfoType_CoreMask, CUR_PROCESS_HANDLE, 0))) return 0;
    return mask;
}

int __wrap___syscall_thread_create(struct __pthread_t** thread, void* (*func)(void*), void* arg, void* stack_addr,
                                   size_t stack_size) {
    int rc = __real___syscall_thread_create(thread, func, arg, stack_addr, stack_size);
    u64 mask = core_mask();
    if (rc != 0 || !*thread || !mask) return rc;
    // The main thread keeps core 0: new threads start from core 1.
    static atomic_uint next = 1;
    unsigned n = atomic_fetch_add(&next, 1) % (unsigned)__builtin_popcountll(mask);
    int core = 0;
    for (; core < 64; core++)
        if ((mask >> core) & 1 && n-- == 0) break;
    // struct __pthread_t starts with libnx's Thread, and Thread with its handle.
    svcSetThreadCoreMask(*(Handle*)*thread, core, (u32)mask);
    return rc;
}

// The cores this process may run on (std has no CPU count on Horizon, and Bevy took that as one).
unsigned int wownx_core_count(void) {
    u64 mask = core_mask();
    return mask ? (unsigned int)__builtin_popcountll(mask) : 1;
}
