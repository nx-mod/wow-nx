// wow-nx: what https.c (haloce-nx's host_https.c) takes from its host, and the updater's own entry (update.c).
#pragma once

#include <stddef.h>

// The log: update.c's sdmc:/wow-nx/update.txt, written before Bevy's log is up.
void host_logf(int priority, const char *format, ...) __attribute__((format(printf, 2, 3)));
#define HOST_LOG_INFO 4
#define HOST_LOG_WARN 5
#define HOST_LOG_ERROR 6

// The longest link followed or handed back (GitHub's signed download links are some 900 characters).
#define HOST_HTTPS_MAXIMUM_URL 2048

// Takes a piece of a body as it arrives (total is its whole length, or -1 if the server did not say); returns 0 to
// stop.
typedef int (*host_https_body)(void *context, const void *data, size_t size, long long total);

// GET url (https://...). A 200's body goes to body, in pieces; redirects are followed if follow_redirects, and the
// last Location seen is copied to location either way. Returns the last status, or -1 with error said.
int host_https_get(const char *url, int follow_redirects, host_https_body body, void *context, char *location,
                   size_t location_size, char *error, size_t error_size);
