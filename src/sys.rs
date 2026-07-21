extern "C" {
    fn raw_open(path: *const u8, flags: i64, mode: i64) -> i64;
    fn raw_read(fd: i64, buf: *mut u8, len: usize) -> i64;
    fn raw_write(fd: i64, buf: *const u8, len: usize) -> i64;
    fn raw_close(fd: i64);
}

const O_RDONLY: i64 = 0;
const O_WRONLY: i64 = 1;
const O_CREAT: i64 = 0o100;
const O_TRUNC: i64 = 0o1000;

fn cpath(path: &str) -> Vec<u8> {
    let mut v = path.as_bytes().to_vec();
    v.push(0);
    v
}

/// Читает файл целиком через ассемблерные syscalls, возвращает обрезанную строку.
pub fn raw_read_to_string(path: &str) -> Option<String> {
    let p = cpath(path);
    unsafe {
        let fd = raw_open(p.as_ptr(), O_RDONLY, 0);
        if fd < 0 {
            return None;
        }
        let mut buf = vec![0u8; 64];
        let n = raw_read(fd, buf.as_mut_ptr(), buf.len());
        raw_close(fd);
        if n < 0 {
            return None;
        }
        buf.truncate(n as usize);
        String::from_utf8(buf).ok().map(|s| s.trim().to_string())
    }
}

/// Пишет строку в файл через ассемблерные syscalls (создаёт/обрезает при необходимости).
pub fn raw_write_str(path: &str, data: &str) -> bool {
    let p = cpath(path);
    unsafe {
        let fd = raw_open(p.as_ptr(), O_WRONLY | O_CREAT | O_TRUNC, 0o644);
        if fd < 0 {
            return false;
        }
        let bytes = data.as_bytes();
        let n = raw_write(fd, bytes.as_ptr(), bytes.len());
        raw_close(fd);
        n as usize == bytes.len()
    }
}

pub fn path_exists(path: &str) -> bool {
    let p = cpath(path);
    unsafe {
        let fd = raw_open(p.as_ptr(), O_RDONLY, 0);
        if fd >= 0 {
            raw_close(fd);
            true
        } else {
            false
        }
    }
}
