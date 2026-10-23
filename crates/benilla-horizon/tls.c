// wow-nx: ELF TLS for Rust on the Switch.
//
// libnx is built with -mtp=soft: its TLS accesses call __aarch64_read_tp(), which returns the thread's TLS pointer
// from libnx's ThreadVars, and nothing loads tpidr_el0. LLVM's AArch64 backend has no soft thread pointer, so Rust's
// thread-locals (the target's has-thread-local) read tpidr_el0. This copies libnx's pointer into tpidr_el0, which EL0
// may write and the kernel keeps per thread: for the main thread before main, and for every pthread (all of Rust's
// threads) before its start routine runs. pthread_create is wrapped at link time (-Wl,--wrap=pthread_create).
#include <pthread.h>
#include <stdlib.h>

extern void* __aarch64_read_tp(void);
extern int __real_pthread_create(pthread_t* thread, const pthread_attr_t* attr, void* (*start)(void*), void* arg);

static inline void set_thread_pointer(void) {
    void* tp = __aarch64_read_tp();
    __asm__ volatile("msr tpidr_el0, %0" ::"r"(tp));
}

__attribute__((constructor(101))) static void main_thread_pointer(void) { set_thread_pointer(); }

typedef struct {
    void* (*start)(void*);
    void* arg;
} Start;

static void* trampoline(void* p) {
    Start s = *(Start*)p;
    free(p);
    set_thread_pointer();
    return s.start(s.arg);
}

int __wrap_pthread_create(pthread_t* thread, const pthread_attr_t* attr, void* (*start)(void*), void* arg) {
    Start* s = malloc(sizeof(Start));
    if (!s) return 11;  // EAGAIN
    s->start = start;
    s->arg = arg;
    int r = __real_pthread_create(thread, attr, trampoline, s);
    if (r != 0) free(s);
    return r;
}
