mod sys;
mod x11;

extern "C" {
    // перенесено в syscalls.asm: чистая арифметика, без строк/аллокаций
    fn octagon_points_asm(cx: i32, cy: i32, half: i32, cut: i32, out: *mut XPoint);
    fn clamp_add_u32(val: u32, step: u32, max: u32) -> u32;
    fn clamp_sub_u32(val: u32, step: u32, floor: u32) -> u32;
}

use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::time::Duration;
use x11::*;

const CANDIDATES: [(&str, &str); 3] = [
    ("/sys/class/backlight/amdgpu_bl0/brightness", "/sys/class/backlight/amdgpu_bl0/max_brightness"),
    ("/sys/class/backlight/amdgpu_bl1/brightness", "/sys/class/backlight/amdgpu_bl1/max_brightness"),
    ("/sys/class/backlight/intel_backlight/brightness", "/sys/class/backlight/intel_backlight/max_brightness"),
];

lazy_static! {
    static ref SAVE_FILE: PathBuf = {
        let home = if let Ok(sudo_user) = env::var("SUDO_USER") {
            format!("/home/{}", sudo_user)
        } else {
            env::var("HOME").unwrap_or_else(|_| ".".into())
        };
        let mut path = PathBuf::from(home);
        path.push(".local");
        path.push("bg");
        path
    };
}

struct Config {
    bright_path: &'static str,
    step_large: u32,
    step_small: u32,
    step_scroll: u32,
}

fn detect_paths() -> (&'static str, &'static str) {
    for (b, m) in CANDIDATES.iter() {
        if sys::path_exists(b) {
            return (b, m);
        }
    }
    CANDIDATES[0]
}

fn save_brightness(val: u32, bright_path: &str) {
    if let Some(parent) = SAVE_FILE.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&*SAVE_FILE, val.to_string());
    sys::raw_write_str(bright_path, &val.to_string());
}

/// Определяет, включён ли уже ночной режим, читая текущую гамму
/// первого попавшегося монитора через `xrandr --verbose`.
fn detect_night_mode() -> bool {
    if let Ok(out) = std::process::Command::new("xrandr").arg("--verbose").output() {
        if let Ok(text) = String::from_utf8(out.stdout) {
            for line in text.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("Gamma:") {
                    // дефолтная гамма -- "1.0:1.0:1.0", любое отклонение считаем ночным режимом
                    return rest.trim() != "1.0:1.0:1.0";
                }
            }
        }
    }
    false
}
fn set_night_mode(enabled: bool) {
    let gamma = if enabled { "1.0:0.9:0.7" } else { "1.0:1.0:1.0" };
    if let Ok(out) = std::process::Command::new("xrandr").output() {
        if let Ok(text) = String::from_utf8(out.stdout) {
            for line in text.lines() {
                // строки подключённых мониторов в выводе xrandr начинаются с
                // "<имя> connected ..."
                if line.contains(" connected") {
                    if let Some(name) = line.split_whitespace().next() {
                        let _ = std::process::Command::new("xrandr")
                            .args(["--output", name, "--gamma", gamma])
                            .status();
                    }
                }
            }
        }
    }
}

const WIN_W: u32 = 340;
const WIN_H: u32 = 32;
const PAD: i32 = 4;
const BAR_H: i32 = 10; // толщина полоски яркости

const ICON_AREA_W: i32 = 30;    // ширина зоны под саму иконку (тело + лучи)
const ICON_GAP: i32 = 10;       // дополнительный отступ между иконкой и полоской
const ICON_RAY_INNER: i32 = 8;  // начало лучей от центра (приближено к телу)
const ICON_RAY_OUTER: i32 = 12; // конец лучей от центра
const ICON_HALF: i32 = 6;       // половина стороны тела иконки
const ICON_CUT: i32 = 4;        // насколько срезаны углы тела
const MOON_CUT_OFFSET: i32 = 5; // на сколько px вырез в теле луны смещён влево от центра

// координаты и размеры самой полоски (после иконки + доп. отступа)
const TRACK_X: i32 = PAD + ICON_AREA_W + ICON_GAP;
const TRACK_W: i32 = WIN_W as i32 - TRACK_X - PAD;
// кликабельная зона иконки (от левого края окна до конца зоны иконки, без зазора)
const ICON_CLICK_W: i32 = PAD + ICON_AREA_W;

// Точки восьмиугольника (квадрат со срезанными углами) с центром (cx, cy).
// Сама арифметика вынесена в octagon_points_asm (syscalls.asm).
fn octagon_points(cx: i32, cy: i32, half: i32, cut: i32) -> [XPoint; 8] {
    let mut pts: [XPoint; 8] = unsafe { std::mem::zeroed() };
    unsafe {
        octagon_points_asm(cx, cy, half, cut, pts.as_mut_ptr());
    }
    pts
}

fn draw(d: *mut Display, win: Window, gc: *mut std::ffi::c_void, val: u32, max: u32, night: bool) {
    unsafe {
        XClearWindow(d, win);
        XSetForeground(d, gc, 0x000000);
        XFillRectangle(d, win, gc, 0, 0, WIN_W, WIN_H);

        // --- иконка "солнце"/"луна" перед полоской ---
        let icon_cx = PAD + ICON_AREA_W / 2;
        let icon_cy = WIN_H as i32 / 2;
        let is_moon = val <= 1;
        // в ночном режиме иконка окрашивается в жёлтый
        let icon_color: u64 = if night { 0xffcc00 } else { 0xffffff };

        XSetForeground(d, gc, icon_color);

        if !is_moon {
            // толстые линии со скруглёнными концами -- как в идеальной версии
            // line_style=0 (LineSolid), cap_style=1 (CapRound), join_style=1 (JoinRound)
            XSetLineAttributes(d, gc, 2, 0, 1, 1);
            // лучи вокруг иконки (углы через 45°, координаты округляем, а не обрезаем,
            // иначе из-за truncation лучи получаются кривыми/несимметричными)
            let ray_inner = ICON_RAY_INNER as f64;
            let ray_outer = ICON_RAY_OUTER as f64;
            for i in 0..8 {
                let angle = (i as f64) * std::f64::consts::PI / 4.0;
                let (sin, cos) = angle.sin_cos();
                let x1 = icon_cx + (cos * ray_inner).round() as i32;
                let y1 = icon_cy + (sin * ray_inner).round() as i32;
                let x2 = icon_cx + (cos * ray_outer).round() as i32;
                let y2 = icon_cy + (sin * ray_outer).round() as i32;
                XDrawLine(d, win, gc, x1, y1, x2, y2);
            }
            // возвращаем толщину линии в 1px, чтобы не влиять на остальную отрисовку
            XSetLineAttributes(d, gc, 1, 0, 0, 0);
        }

        // тело иконки -- восьмиугольник (одинаковый для солнца и луны)
        let mut body = octagon_points(icon_cx, icon_cy, ICON_HALF, ICON_CUT);
        XFillPolygon(d, win, gc, body.as_mut_ptr(), body.len() as i32, 2, 0);

        if is_moon {
            // вырез внутри тела луны -- такой же восьмиугольник, но цветом фона
            // и смещённый вправо, из-за чего слева остаётся тонкий серп
            XSetForeground(d, gc, 0x000000);
            let mut cutout = octagon_points(icon_cx + MOON_CUT_OFFSET, icon_cy, ICON_HALF, ICON_CUT);
            XFillPolygon(d, win, gc, cutout.as_mut_ptr(), cutout.len() as i32, 2, 0);
        }

        // --- полоска яркости ---
        let bar_y = WIN_H as i32 / 2 - BAR_H / 2;

        XSetForeground(d, gc, 0x404040);
        XFillRectangle(d, win, gc, TRACK_X, bar_y, TRACK_W as u32, BAR_H as u32);

        let fill_w = ((TRACK_W as u64 * val as u64) / max.max(1) as u64) as u32;
        XSetForeground(d, gc, 0xffffff);
        XFillRectangle(d, win, gc, TRACK_X, bar_y, fill_w, BAR_H as u32);

        XFlush(d);
    }
}

fn main() {
    let (bright_path, max_bright_path) = detect_paths();
    let max_bright: u32 = sys::raw_read_to_string(max_bright_path)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9600);
    let cfg = Config {
        bright_path,
        step_large: (max_bright / 20).max(1),
        step_small: (max_bright / 100).max(1),
        step_scroll: (max_bright as f64 * 0.05).round().max(1.0) as u32,
    };
    let mut current: u32 = sys::raw_read_to_string(cfg.bright_path)
        .and_then(|s| s.parse().ok())
        .unwrap_or(max_bright / 2);

    unsafe {
        let d = XOpenDisplay(std::ptr::null());
        if d.is_null() {
            eprintln!("не удалось открыть X display");
            return;
        }
        let screen = XDefaultScreen(d);
        let root = XDefaultRootWindow(d);
        let screen_w = XDisplayWidth(d, screen);
        let screen_h = XDisplayHeight(d, screen);

        // позиция курсора
        let (mut rx, mut ry, mut wx, mut wy) = (0, 0, 0, 0);
        let mut mask: u32 = 0;
        let (mut root_ret, mut child_ret) = (0u64, 0u64);
        XQueryPointer(d, root, &mut root_ret, &mut child_ret, &mut rx, &mut ry, &mut wx, &mut wy, &mut mask);

        // окно всегда внутри границ экрана
        let win_x = (rx - WIN_W as i32 / 2).clamp(0, (screen_w - WIN_W as i32).max(0));
        let win_y = (ry - WIN_H as i32 - 10).clamp(0, (screen_h - WIN_H as i32).max(0));

        // border_width = 0, чтобы не было обводки вокруг окна
        let win = XCreateSimpleWindow(d, root, win_x, win_y, WIN_W, WIN_H, 0, 0x000000, 0x000000);

        // override-redirect, чтобы bspwm не тайлил окно
        let mut attrs: XSetWindowAttributes = std::mem::zeroed();
        attrs.override_redirect = 1;
        XChangeWindowAttributes(d, win, CW_OVERRIDE_REDIRECT, &mut attrs);

        XSelectInput(
            d,
            win,
            EXPOSURE_MASK | KEY_PRESS_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK | POINTER_MOTION_MASK,
        );
        XMapWindow(d, win);
        XFlush(d);

        // Рисуем сразу же, чтобы окно не висело пустым чёрным прямоугольником
        // пока идут попытки захвата указателя/клавиатуры ниже.
        let q_keycode = XKeysymToKeycode(d, XK_Q); // физическая клавиша Q/Й общая
        let gc = XDefaultGC(d, screen);
        let mut night_mode = detect_night_mode();
        draw(d, win, gc, current, max_bright, night_mode);

        // Сразу после клика по polybar X-сервер может ещё на несколько мс держать
        // системный grab, связанный с обработкой самого этого клика. Поэтому
        // пытаемся захватить указатель несколько раз, пока это не удастся.
        // Окно уже нарисовано выше, так что эти попытки не задерживают появление UI.
        let mut pointer_grabbed = false;
        for _ in 0..50 {
            let res = XGrabPointer(
                d,
                root,
                1,
                (BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK | POINTER_MOTION_MASK) as u32,
                GRAB_MODE_ASYNC,
                GRAB_MODE_ASYNC,
                0,
                0,
                CURRENT_TIME,
            );
            if res == 0 {
                pointer_grabbed = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if !pointer_grabbed {
            eprintln!("bg: не удалось захватить указатель мыши -- закрытие по клику мимо окна может не сработать");
        }

        let mut keyboard_grabbed = false;
        for _ in 0..50 {
            let res = XGrabKeyboard(d, win, 1, GRAB_MODE_ASYNC, GRAB_MODE_ASYNC, CURRENT_TIME);
            if res == 0 {
                keyboard_grabbed = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if !keyboard_grabbed {
            eprintln!("bg: не удалось захватить клавиатуру");
        }

        // true только между ButtonPress внутри окна и следующим ButtonRelease
        let mut dragging = false;

        'outer: loop {
            let mut ev: XEvent = std::mem::zeroed();
            XNextEvent(d, &mut ev);

            match ev.type_ {
                2 => {
                    // KeyPress
                    let mut kev = ev.key;
                    let sym = XLookupKeysym(&mut kev, 0);
                    if kev.keycode as u8 == q_keycode || sym == XK_ESCAPE {
                        break 'outer;
                    } else if sym == XK_RIGHT {
                        current = clamp_add_u32(current, cfg.step_small, max_bright);
                    } else if sym == XK_LEFT {
                        current = clamp_sub_u32(current, cfg.step_small, 1);
                    } else if sym == XK_UP {
                        current = clamp_add_u32(current, cfg.step_large, max_bright);
                    } else if sym == XK_DOWN {
                        current = clamp_sub_u32(current, cfg.step_large, 1);
                    } else {
                        continue;
                    }
                    save_brightness(current, cfg.bright_path);
                    draw(d, win, gc, current, max_bright, night_mode);
                }
                4 => {
                    // ButtonPress. Координаты всегда берём относительно root (x_root/y_root) --
                    // они не зависят от того, в какое окно X-сервер формально "попал" кликом,
                    // поэтому проверка границ надёжна даже при клике по чужому окну (polybar и т.п.)
                    let bev = ev.button;

                    // Колёсико мыши (button 4 = вверх, button 5 = вниз) работает
                    // в любой точке экрана, т.к. указатель захвачен глобально,
                    // и не должно закрывать окно.
                    if bev.button == 4 {
                        current = clamp_add_u32(current, cfg.step_scroll, max_bright);
                        save_brightness(current, cfg.bright_path);
                        draw(d, win, gc, current, max_bright, night_mode);
                        continue;
                    } else if bev.button == 5 {
                        current = clamp_sub_u32(current, cfg.step_scroll, 1);
                        save_brightness(current, cfg.bright_path);
                        draw(d, win, gc, current, max_bright, night_mode);
                        continue;
                    }

                    let local_x = bev.x_root - win_x;
                    let local_y = bev.y_root - win_y;

                    if local_x < 0 || local_x > WIN_W as i32 || local_y < 0 || local_y > WIN_H as i32 {
                        break 'outer;
                    }

                    // клик по территории иконки солнца/луны -- переключаем ночной режим,
                    // не трогая яркость и не начиная перетаскивание полоски
                    if local_x <= ICON_CLICK_W {
                        night_mode = !night_mode;
                        set_night_mode(night_mode);
                        draw(d, win, gc, current, max_bright, night_mode);
                        continue;
                    }

                    dragging = true;
                    let rel = (local_x - TRACK_X).clamp(0, TRACK_W);
                    current = ((rel as u64 * max_bright as u64) / TRACK_W as u64).max(1) as u32;
                    save_brightness(current, cfg.bright_path);
                    draw(d, win, gc, current, max_bright, night_mode);
                }
                5 => {
                    // ButtonRelease — прекращаем тянуть слайдер
                    dragging = false;
                }
                6 => {
                    // MotionNotify — двигаем яркость только пока зажата кнопка
                    if dragging {
                        let mev = ev.motion;
                        let local_x = mev.x_root - win_x;
                        let rel = (local_x - TRACK_X).clamp(0, TRACK_W);
                        current = ((rel as u64 * max_bright as u64) / TRACK_W as u64).max(1) as u32;
                        save_brightness(current, cfg.bright_path);
                        draw(d, win, gc, current, max_bright, night_mode);
                    }
                }
                _ => {}
            }
        }

        XUngrabKeyboard(d, CURRENT_TIME);
        XUngrabPointer(d, CURRENT_TIME);
        XCloseDisplay(d);
    }

    save_brightness(current, cfg.bright_path);
}
