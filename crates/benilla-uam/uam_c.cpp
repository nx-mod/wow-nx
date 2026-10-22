// wow-nx: uam_api.h behind a C ABI for Rust (src/lib.rs). Called from one thread only (uam is not reentrant).
#include <cstdlib>
#include <cstring>

#include "uam_api.h"

extern "C" int benilla_uam_compilar(int etapa, const char* glsl, unsigned char** dksh, size_t* tamano, char** registro) {
    static bool iniciado = false;
    if (!iniciado) {
        uam::init(true);
        iniciado = true;
    }
    uam::Result r = uam::compile(static_cast<uam::Stage>(etapa), glsl);
    *dksh = nullptr;
    *tamano = 0;
    if (r.ok && !r.dksh.empty()) {
        *dksh = static_cast<unsigned char*>(std::malloc(r.dksh.size()));
        std::memcpy(*dksh, r.dksh.data(), r.dksh.size());
        *tamano = r.dksh.size();
    }
    *registro = strdup(r.log.c_str());
    return r.ok ? 1 : 0;
}

extern "C" void benilla_uam_liberar(void* p) { std::free(p); }
