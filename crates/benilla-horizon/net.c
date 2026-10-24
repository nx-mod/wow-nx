// wow-nx: the network. newlib's socket calls, and so Rust std's TcpStream and DNS lookups, go through libnx's BSD
// sockets, which answer nothing until socketInitialize opens the session (bsd:u, sfdnsres): without it the logon
// lookup and connect failed, and the client showed "disconnected" and retried.
#include <switch.h>

// 0 on success, else the libnx result.
unsigned int wownx_network_init(void) {
    return socketInitializeDefault();
}
