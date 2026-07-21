use std::os::raw::{c_char, c_int, c_uint, c_ulong, c_void};

pub type Display = c_void;
pub type Window = c_ulong;

pub const CW_OVERRIDE_REDIRECT: c_ulong = 1 << 9;
pub const BUTTON_PRESS_MASK: i64 = 1 << 2;
pub const BUTTON_RELEASE_MASK: i64 = 1 << 3;
pub const POINTER_MOTION_MASK: i64 = 1 << 6;
pub const KEY_PRESS_MASK: i64 = 1 << 0;
pub const EXPOSURE_MASK: i64 = 1 << 15;
pub const GRAB_MODE_ASYNC: c_int = 1;
pub const CURRENT_TIME: c_ulong = 0;

pub const XK_LEFT: c_ulong = 0xff51;
pub const XK_UP: c_ulong = 0xff52;
pub const XK_RIGHT: c_ulong = 0xff53;
pub const XK_DOWN: c_ulong = 0xff54;
pub const XK_ESCAPE: c_ulong = 0xff1b;
pub const XK_Q: c_ulong = 0x71;

#[repr(C)]
pub struct XSetWindowAttributes {
    pub background_pixmap: c_ulong,
    pub background_pixel: c_ulong,
    pub border_pixmap: c_ulong,
    pub border_pixel: c_ulong,
    pub bit_gravity: c_int,
    pub win_gravity: c_int,
    pub backing_store: c_int,
    pub backing_planes: c_ulong,
    pub backing_pixel: c_ulong,
    pub save_under: c_int,
    pub event_mask: i64,
    pub do_not_propagate_mask: i64,
    pub override_redirect: c_int,
    pub colormap: c_ulong,
    pub cursor: c_ulong,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XKeyEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int, pub y: c_int,
    pub x_root: c_int, pub y_root: c_int,
    pub state: c_uint,
    pub keycode: c_uint,
    pub same_screen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XButtonEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int, pub y: c_int,
    pub x_root: c_int, pub y_root: c_int,
    pub state: c_uint,
    pub button: c_uint,
    pub same_screen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XMotionEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int, pub y: c_int,
    pub x_root: c_int, pub y_root: c_int,
    pub state: c_uint,
    pub is_hint: i8,
    pub same_screen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union XEvent {
    pub type_: c_int,
    pub key: XKeyEvent,
    pub button: XButtonEvent,
    pub motion: XMotionEvent,
    pub pad: [u8; 192],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XPoint {
    pub x: i16,
    pub y: i16,
}

#[link(name = "X11")]
extern "C" {
    pub fn XOpenDisplay(name: *const c_char) -> *mut Display;
    pub fn XCloseDisplay(d: *mut Display) -> c_int;
    pub fn XDefaultScreen(d: *mut Display) -> c_int;
    pub fn XDefaultRootWindow(d: *mut Display) -> Window;
    pub fn XDisplayWidth(d: *mut Display, screen: c_int) -> c_int;
    pub fn XDisplayHeight(d: *mut Display, screen: c_int) -> c_int;
    pub fn XCreateSimpleWindow(d: *mut Display, parent: Window, x: c_int, y: c_int, w: c_uint, h: c_uint, bw: c_uint, border: c_ulong, bg: c_ulong) -> Window;
    pub fn XChangeWindowAttributes(d: *mut Display, w: Window, valuemask: c_ulong, attrs: *mut XSetWindowAttributes) -> c_int;
    pub fn XSelectInput(d: *mut Display, w: Window, mask: i64) -> c_int;
    pub fn XMapWindow(d: *mut Display, w: Window) -> c_int;
    pub fn XFlush(d: *mut Display) -> c_int;
    pub fn XNextEvent(d: *mut Display, ev: *mut XEvent) -> c_int;
    pub fn XQueryPointer(d: *mut Display, w: Window, root_ret: *mut Window, child_ret: *mut Window, root_x: *mut c_int, root_y: *mut c_int, win_x: *mut c_int, win_y: *mut c_int, mask: *mut c_uint) -> c_int;
    pub fn XGrabPointer(d: *mut Display, gw: Window, owner_events: c_int, mask: c_uint, pm: c_int, km: c_int, confine: Window, cursor: c_ulong, time: c_ulong) -> c_int;
    pub fn XGrabKeyboard(d: *mut Display, gw: Window, owner_events: c_int, pm: c_int, km: c_int, time: c_ulong) -> c_int;
    pub fn XUngrabPointer(d: *mut Display, time: c_ulong) -> c_int;
    pub fn XUngrabKeyboard(d: *mut Display, time: c_ulong) -> c_int;
    pub fn XDefaultGC(d: *mut Display, screen: c_int) -> *mut c_void;
    pub fn XSetForeground(d: *mut Display, gc: *mut c_void, pixel: c_ulong) -> c_int;
    pub fn XFillRectangle(d: *mut Display, drawable: Window, gc: *mut c_void, x: c_int, y: c_int, w: c_uint, h: c_uint) -> c_int;
    #[allow(dead_code)]
    pub fn XFillArc(d: *mut Display, drawable: Window, gc: *mut c_void, x: c_int, y: c_int, w: c_uint, h: c_uint, angle1: c_int, angle2: c_int) -> c_int;
    pub fn XFillPolygon(d: *mut Display, drawable: Window, gc: *mut c_void, points: *mut XPoint, npoints: c_int, shape: c_int, mode: c_int) -> c_int;
    pub fn XDrawLine(d: *mut Display, drawable: Window, gc: *mut c_void, x1: c_int, y1: c_int, x2: c_int, y2: c_int) -> c_int;
    pub fn XSetLineAttributes(d: *mut Display, gc: *mut c_void, line_width: c_uint, line_style: c_int, cap_style: c_int, join_style: c_int) -> c_int;
    pub fn XClearWindow(d: *mut Display, w: Window) -> c_int;
    pub fn XLookupKeysym(ev: *mut XKeyEvent, index: c_int) -> c_ulong;
    pub fn XKeysymToKeycode(d: *mut Display, keysym: c_ulong) -> u8;
}
