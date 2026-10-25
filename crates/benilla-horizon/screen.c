// wow-nx: the loading picture at launch. Until a process presents its first frame, the Switch shows the HOME menu's
// launch screen (white with the white theme), and the game's first frame comes after seconds of loading. One frame
// through libnx's software framebuffer replaces it: sdmc:/wow-nx/loading.rgba (1280x720 RGBA, made from the client's
// own loading screen art), or black without it. The framebuffer stays connected, so the picture stays up through the
// seconds of loading (a closed one left the layer black); the deko3d backend releases it just before it makes its
// swapchain (wownx_release_start_screen), and keeps the picture through the start's held frames (StartPicture).
#include <stdio.h>
#include <string.h>
#include <switch.h>

enum { WIDTH = 1280, HEIGHT = 720 };

static Framebuffer fb;
static bool fb_open;

void wownx_black_screen(void) {
    NWindow* win = nwindowGetDefault();
    if (R_FAILED(framebufferCreate(&fb, win, WIDTH, HEIGHT, PIXEL_FORMAT_RGBA_8888, 2))) return;
    fb_open = true;
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
}

void wownx_update_offer(void);

// The window, back for deko3d's swapchain: called by the backend before dkSwapchainCreate. A newer release the updater
// found meanwhile is offered first (update.c), on the window just freed.
void wownx_release_start_screen(void) {
    if (fb_open) {
        framebufferClose(&fb);
        fb_open = false;
    }
    wownx_update_offer();
}
