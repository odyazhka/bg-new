section .text
global raw_open
global raw_read
global raw_write
global raw_close
global octagon_points_asm
global clamp_add_u32
global clamp_sub_u32

; i64 raw_open(const char *path, i64 flags, i64 mode)
; System V ABI: rdi=path, rsi=flags, rdx=mode -- уже на месте для syscall
raw_open:
    mov rax, 2          ; sys_open
    syscall
    ret

; i64 raw_read(i64 fd, u8 *buf, u64 len)
; rdi=fd, rsi=buf, rdx=len
raw_read:
    mov rax, 0          ; sys_read
    syscall
    ret

; i64 raw_write(i64 fd, const u8 *buf, u64 len)
; rdi=fd, rsi=buf, rdx=len
raw_write:
    mov rax, 1          ; sys_write
    syscall
    ret

; void raw_close(i64 fd)
; rdi=fd
raw_close:
    mov rax, 3          ; sys_close
    syscall
    ret

; ----------------------------------------------------------------------
; Ниже -- чистая целочисленная арифметика без сисколлов, перенесена из
; main.rs как есть (те же формулы), т.к. не зависит от String/Vec/Option
; и легко ложится на регистры.
; ----------------------------------------------------------------------

; void octagon_points_asm(i32 cx, i32 cy, i32 half, i32 cut, XPoint *out)
; XPoint { x: i16, y: i16 } -- 8 точек подряд, 4 байта каждая, итого 32 байта в out.
; System V ABI: edi=cx, esi=cy, edx=half, ecx=cut, r8=out
octagon_points_asm:
    push rbx
    mov r9d, edi        ; cx
    mov r10d, esi        ; cy
    mov r11d, edx        ; half
    mov ebx, ecx          ; cut

    ; point0: x = cx-half+cut, y = cy-half
    mov eax, r9d
    sub eax, r11d
    add eax, ebx
    mov word [r8+0], ax
    mov eax, r10d
    sub eax, r11d
    mov word [r8+2], ax

    ; point1: x = cx+half-cut, y = cy-half
    mov eax, r9d
    add eax, r11d
    sub eax, ebx
    mov word [r8+4], ax
    mov eax, r10d
    sub eax, r11d
    mov word [r8+6], ax

    ; point2: x = cx+half, y = cy-half+cut
    mov eax, r9d
    add eax, r11d
    mov word [r8+8], ax
    mov eax, r10d
    sub eax, r11d
    add eax, ebx
    mov word [r8+10], ax

    ; point3: x = cx+half, y = cy+half-cut
    mov eax, r9d
    add eax, r11d
    mov word [r8+12], ax
    mov eax, r10d
    add eax, r11d
    sub eax, ebx
    mov word [r8+14], ax

    ; point4: x = cx+half-cut, y = cy+half
    mov eax, r9d
    add eax, r11d
    sub eax, ebx
    mov word [r8+16], ax
    mov eax, r10d
    add eax, r11d
    mov word [r8+18], ax

    ; point5: x = cx-half+cut, y = cy+half
    mov eax, r9d
    sub eax, r11d
    add eax, ebx
    mov word [r8+20], ax
    mov eax, r10d
    add eax, r11d
    mov word [r8+22], ax

    ; point6: x = cx-half, y = cy+half-cut
    mov eax, r9d
    sub eax, r11d
    mov word [r8+24], ax
    mov eax, r10d
    add eax, r11d
    sub eax, ebx
    mov word [r8+26], ax

    ; point7: x = cx-half, y = cy-half+cut
    mov eax, r9d
    sub eax, r11d
    mov word [r8+28], ax
    mov eax, r10d
    sub eax, r11d
    add eax, ebx
    mov word [r8+30], ax

    pop rbx
    ret

; u32 clamp_add_u32(u32 val, u32 step, u32 max)
; edi=val, esi=step, edx=max  -->  min(val + step, max)
clamp_add_u32:
    mov eax, edi
    add eax, esi
    cmp eax, edx
    jbe .ok_add
    mov eax, edx
.ok_add:
    ret

; u32 clamp_sub_u32(u32 val, u32 step, u32 floor)
; edi=val, esi=step, edx=floor  -->  max(val.saturating_sub(step), floor)
clamp_sub_u32:
    mov eax, edi
    sub eax, esi
    jnc .check_floor      ; unsigned borrow => underflow
    xor eax, eax
.check_floor:
    cmp eax, edx
    jae .ok_sub
    mov eax, edx
.ok_sub:
    ret

section .note.GNU-stack noalloc noexec nowrite
