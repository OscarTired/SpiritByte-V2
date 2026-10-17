/* Test-only Wayland virtual pointer. The cursor is positioned by the runner.
 * Protocol: wlr-protocols/unstable/wlr-virtual-pointer-unstable-v1.xml, version 1.
 * Compiled into a temporary directory, never included in the installed client.
 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>

static const struct wl_message pointer_requests[] = {
    {"motion", "uff", NULL}, {"motion_absolute", "uuuuu", NULL},
    {"button", "uuu", NULL}, {"axis", "uuf", NULL}, {"frame", "", NULL},
    {"axis_source", "u", NULL}, {"axis_stop", "uu", NULL},
    {"axis_discrete", "uufi", NULL}, {"destroy", "", NULL},
};
static const struct wl_interface pointer_interface = {
    "zwlr_virtual_pointer_v1", 1, 9, pointer_requests, 0, NULL
};
static const struct wl_interface *create_types[] = {&wl_seat_interface, &pointer_interface};
static const struct wl_message manager_requests[] = {
    {"create_virtual_pointer", "?on", create_types}, {"destroy", "", NULL},
};
static const struct wl_interface manager_interface = {
    "zwlr_virtual_pointer_manager_v1", 1, 2, manager_requests, 0, NULL
};
static struct wl_proxy *manager;
static void global(void *data, struct wl_registry *registry, uint32_t name, const char *interface, uint32_t version) {
    (void)data; (void)version;
    if (!strcmp(interface, manager_interface.name))
        manager = wl_registry_bind(registry, name, &manager_interface, 1);
}
static void removed(void *data, struct wl_registry *registry, uint32_t name) { (void)data; (void)registry; (void)name; }
static const struct wl_registry_listener listener = {global, removed};
static uint32_t milliseconds(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint32_t)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);
}
int main(void) {
    struct wl_display *display = wl_display_connect(NULL);
    if (!display) { fputs("Cannot connect to Wayland\n", stderr); return 1; }
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &listener, NULL);
    wl_display_roundtrip(display);
    if (!manager) { fputs("Compositor has no virtual pointer protocol\n", stderr); return 1; }
    struct wl_proxy *pointer = wl_proxy_marshal_flags(manager, 0, &pointer_interface, 1, 0, NULL, NULL);
    wl_display_roundtrip(display);
    wl_proxy_marshal_flags(pointer, 2, NULL, 1, 0, milliseconds(), 272u, 1u);
    wl_proxy_marshal_flags(pointer, 4, NULL, 1, 0);
    wl_display_roundtrip(display);
    usleep(80000);
    wl_proxy_marshal_flags(pointer, 2, NULL, 1, 0, milliseconds(), 272u, 0u);
    wl_proxy_marshal_flags(pointer, 4, NULL, 1, 0);
    wl_display_roundtrip(display);
    wl_proxy_marshal_flags(pointer, 8, NULL, 1, WL_MARSHAL_FLAG_DESTROY);
    wl_proxy_marshal_flags(manager, 1, NULL, 1, WL_MARSHAL_FLAG_DESTROY);
    wl_registry_destroy(registry);
    wl_display_roundtrip(display);
    wl_display_disconnect(display);
    return 0;
}
