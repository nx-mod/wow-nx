// wow-nx: uam_api.h behind a C ABI for Rust (src/lib.rs). Called from one thread only (uam is not reentrant).
#include <cstdlib>
#include <cstring>

#include "uam_api.h"

extern "C" int benilla_uam_compile(int stage, const char* glsl, unsigned char** dksh, size_t* size, char** log) {
    static bool initialized = false;
    if (!initialized) {
        uam::init(true);
        initialized = true;
    }
    uam::Result r = uam::compile(static_cast<uam::Stage>(stage), glsl);
    *dksh = nullptr;
    *size = 0;
    if (r.ok && !r.dksh.empty()) {
        *dksh = static_cast<unsigned char*>(std::malloc(r.dksh.size()));
        std::memcpy(*dksh, r.dksh.data(), r.dksh.size());
        *size = r.dksh.size();
    }
    *log = strdup(r.log.c_str());
    return r.ok ? 1 : 0;
}

extern "C" void benilla_uam_free(void* p) { std::free(p); }
