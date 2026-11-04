//! Fullscreen that stays on the monitor the window is on.
//!
//! On Windows the stock fullscreen always jumps to the primary monitor. Here the window
//! becomes a borderless popup that covers exactly the monitor it is currently on, and
//! goes back to its old style and position when fullscreen is turned off.
//! Other systems use the regular macroquad fullscreen.

#[cfg(all(windows, target_pointer_width = "64"))]
mod imp {
    use std::cell::Cell;
    use windows_sys::Win32::Foundation::{HWND, RECT};
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId,
        SWP_FRAMECHANGED, SWP_NOOWNERZORDER, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos,
        WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
    };

    thread_local! {
        /// Window style and rectangle saved before going fullscreen.
        static SAVED: Cell<Option<(isize, RECT)>> = const { Cell::new(None) };
    }

    /// The game window: the active window of this thread, or the foreground window
    /// when it belongs to this process.
    fn game_window() -> Option<HWND> {
        // SAFETY: plain Win32 queries with no pointers handed over except a local u32.
        unsafe {
            let h = GetActiveWindow();
            if !h.is_null() {
                return Some(h);
            }
            let f = GetForegroundWindow();
            if f.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(f, &mut pid);
            (pid == GetCurrentProcessId()).then_some(f)
        }
    }

    /// Returns false when the window could not be found (the caller falls back).
    pub fn set(on: bool) -> bool {
        let Some(hwnd) = game_window() else {
            return false;
        };
        // SAFETY: `hwnd` is a live window of this process; every struct passed by pointer
        // is a local that outlives the call, with `cbSize` set where the API requires it.
        unsafe {
            if on {
                if SAVED.with(|s| s.get()).is_some() {
                    return true;
                }
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                let mut rect: RECT = std::mem::zeroed();
                GetWindowRect(hwnd, &mut rect);
                let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(mon, &mut info) == 0 {
                    return false;
                }
                SAVED.with(|s| s.set(Some((style, rect))));
                let popup =
                    (style & !(WS_OVERLAPPEDWINDOW as isize)) | (WS_POPUP | WS_VISIBLE) as isize;
                SetWindowLongPtrW(hwnd, GWL_STYLE, popup);
                let m = info.rcMonitor;
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    m.left,
                    m.top,
                    m.right - m.left,
                    m.bottom - m.top,
                    SWP_FRAMECHANGED | SWP_NOOWNERZORDER,
                );
            } else if let Some((style, r)) = SAVED.with(|s| s.take()) {
                SetWindowLongPtrW(hwnd, GWL_STYLE, style);
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOOWNERZORDER,
                );
            }
        }
        true
    }
}

/// Turns fullscreen on or off on the monitor that currently holds the window.
pub fn set_fullscreen(on: bool) {
    #[cfg(all(windows, target_pointer_width = "64"))]
    if imp::set(on) {
        return;
    }
    macroquad::window::set_fullscreen(on);
}
