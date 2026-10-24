// wow-nx: the loading picture at launch. Until a process presents its first frame, the Switch shows the HOME menu's
// launch screen (white with the white theme), and the game's first frame comes after seconds of loading. One frame
// through libnx's software framebuffer replaces it: sdmc:/wow-nx/loading.rgba (1280x720 RGBA, made from the client's
// own loading screen art), or black without it. The framebuffer is closed again, leaving the window to deko3d's
// swapchain, which keeps the picture through the start's held frames (the deko3d backend's StartPicture).
#include <stdio.h>
#include <string.h>
#include <switch.h>

enum { WIDTH = 1280, HEIGHT = 720 };

void wownx_black_screen(void) {
    NWindow* win = nwindowGetDefault();
    Framebuffer fb;
    if (R_FAILED(framebufferCreate(&fb, win, WIDTH, HEIGHT, PIXEL_FORMAT_RGBA_8888, 2))) return;
    if (R_SUCCEEDED(framebufferMakeLinear(&fb))) {
        u32 stride = 0;
        u8* pixels = framebufferBegin(&fb, &stride);
        if (pixels) {
            memset(pixels, 0, (size_t)stride * HEIGHT);
            FILE* f = fopen("sdmc:/wow-nx/loading.rgba", "rb");
            if (f) {
                for (int y = 0; y < HEIGHT; y++)
                    if (fread(pixels + (size_t)y * stride, 4, WIDTH, f) != WIDTH) break;
                fclose(f);
            }
        }
        framebufferEnd(&fb);
    }
    framebufferClose(&fb);
}
