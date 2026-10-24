// wow-nx: a black screen at launch. Until a process presents its first frame, the Switch shows the HOME menu's
// launch screen (white with the white theme), and the game's first frame comes after seconds of loading. One black
// frame through libnx's software framebuffer replaces it; the framebuffer is closed again, leaving the window to
// deko3d's swapchain.
#include <string.h>
#include <switch.h>

void wownx_black_screen(void) {
    NWindow* win = nwindowGetDefault();
    Framebuffer fb;
    if (R_FAILED(framebufferCreate(&fb, win, 1280, 720, PIXEL_FORMAT_RGBA_8888, 2))) return;
    if (R_SUCCEEDED(framebufferMakeLinear(&fb))) {
        u32 stride = 0;
        void* pixels = framebufferBegin(&fb, &stride);
        if (pixels) memset(pixels, 0, (size_t)stride * 720);
        framebufferEnd(&fb);
    }
    framebufferClose(&fb);
}
