use x11::xlib;
use x11::xtest;
use crate::xcommon::X;
use crate::xcommon::Area;
use std::mem::MaybeUninit;

pub fn get_key_press(x: &X) -> Option<(u32, u32)> {
    unsafe {
        let mut ev: xlib::XEvent = MaybeUninit::zeroed().assume_init();

        loop {
            if xlib::XNextEvent(x.dpy, &mut ev) != 0 { return None; }

            if ev.type_ == xlib::KeyPress { return Some((ev.key.keycode, ev.key.state)); }

            xlib::XRaiseWindow(x.dpy, x.window);
        }
    }
}

pub fn move_cursor_and_click(x: &X, button: i32, ox: i32, oy: i32, no_release: bool) {
    unsafe {
        xlib::XLowerWindow(x.dpy, x.window);

        if ox >= 0 && oy >= 0 {
            xlib::XWarpPointer(x.dpy, 0, x.rootwin, 0, 0, 0, 0, ox, oy);
        }

        let b: Result<u32, _> = button.try_into();

        match b {
            Ok(val) => {
                xtest::XTestFakeButtonEvent(x.dpy, val, 1, 0);
                xlib::XSync(x.dpy, 0);

                if !no_release {
                    xtest::XTestFakeButtonEvent(x.dpy, val, 0, 0);
                    xlib::XSync(x.dpy, 0);
                }
            },
            _ => {}
        }

        xlib::XRaiseWindow(x.dpy, x.window);
    }
}

pub fn move_cursor_edge_and_click(x: &X, area: &Area, segment: u32, button: i32, no_release: bool) {
    let ox = match segment % 3 {
        0 => area.x,
        1 => area.x + area.w / 2,
        2 => area.x + area.w - 1,
        _ => return
    };
    
    let oy = match segment / 3 {
        0 => area.y,
        1 => area.y + area.h / 2,
        2 => area.y + area.h - 1,
        _ => return
    };

    move_cursor_and_click(x, button, ox, oy, no_release);
}
