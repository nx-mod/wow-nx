// wow-nx: the Switch's system keyboard (the swkbd applet), for the text boxes: benilla-app's switch.rs opens it on a
// tap on a box and types what it returns into the box.
#include <string.h>
#include <switch.h>

// Shows the keyboard over the game, which waits. `initial` is the box's text (not shown for a password box); the
// result goes to `out`, NUL-terminated. 1 if the player pressed OK, 0 if they cancelled or it failed.
int wownx_keyboard(const char* initial, int password, char* out, size_t size) {
    SwkbdConfig kbd;
    if (R_FAILED(swkbdCreate(&kbd, 0))) return 0;
    if (password)
        swkbdConfigMakePresetPassword(&kbd);
    else {
        swkbdConfigMakePresetDefault(&kbd);
        swkbdConfigSetInitialText(&kbd, initial);
    }
    memset(out, 0, size);
    Result rc = swkbdShow(&kbd, out, size);
    swkbdClose(&kbd);
    return R_SUCCEEDED(rc);
}
