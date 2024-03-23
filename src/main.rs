use x11::xlib;
use x11::xtest;
use std::mem::MaybeUninit;

struct X {
    dpy: *mut xlib::Display,
    rootwin: xlib::Window,
    window: xlib::Window,
    colormap: xlib::Colormap,
    gc: xlib::GC,
    w: i32,
    h: i32
}

fn start_x() -> Option<X> {
    unsafe {
        // thx https://stackoverflow.com/questions/21780789/x11-draw-on-overlay-window

		let dpy = xlib::XOpenDisplay(std::ptr::null());
		let screen = xlib::XDefaultScreen(dpy);
		let rootwin = xlib::XRootWindow(dpy, screen);

        let mut attrs: xlib::XSetWindowAttributes = MaybeUninit::zeroed().assume_init();
        attrs.override_redirect = 1;

        let mut vinfo: xlib::XVisualInfo = MaybeUninit::zeroed().assume_init();

        if xlib::XMatchVisualInfo(dpy, screen, 32, xlib::TrueColor, &mut vinfo) == 0 {
            println!("No visual found supporting 32 bit color");
            return None;
        }

        attrs.colormap = xlib::XCreateColormap(dpy, rootwin, vinfo.visual, xlib::AllocNone);
        attrs.background_pixel = 0;
        attrs.border_pixel = 0;

        let w = xlib::XDisplayWidth(dpy, screen);
        let h = xlib::XDisplayHeight(dpy, screen);

        let overlay: xlib::Window = xlib::XCreateWindow(
            dpy, rootwin,
            0, 0, w.try_into().unwrap(), h.try_into().unwrap(), 0,
            vinfo.depth,
            1, // = InputOutput
            vinfo.visual,
            xlib::CWOverrideRedirect | xlib::CWColormap | xlib::CWBackPixel | xlib::CWBorderPixel,
            &mut attrs);

        // the following was lifted from suckless' slock

        let mut grab = xlib::GrabFrozen;

        for _i in 0..10 {
            grab = xlib::XGrabKeyboard(dpy, rootwin, 1, xlib::GrabModeAsync, xlib::GrabModeAsync, xlib::CurrentTime);

            if grab == xlib::GrabSuccess {
                xlib::XMapRaised(dpy, overlay);
                xlib::XSelectInput(dpy, rootwin, xlib::KeyPressMask);
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if grab != xlib::GrabSuccess {
            return None;
        }

        let gc = xlib::XCreateGC(dpy, overlay, 0, std::ptr::null_mut());

        Some(X {
            dpy,
            rootwin,
            window: overlay,
            colormap: attrs.colormap,
            gc,
            w,
            h
        })
    }
}

#[derive(Clone)]
struct Area {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

unsafe fn draw_dotted_line_horiz(x: &X, ox: i32, oy: i32, w: i32) {
    for off in 0..2 {
        let mut _x = ox + off;

        xlib::XSetForeground(x.dpy, x.gc, if off == 0 {
            0x80000000
        } else {
            0x80FFFFFF
        });

        while _x < (ox + w) {
            xlib::XDrawPoint(x.dpy, x.window, x.gc, _x, oy);
            _x += 2;
        }
    }
}

unsafe fn draw_dotted_line_vert(x: &X, ox: i32, oy: i32, h: i32) {
    for off in 0..2 {
        let mut _y = oy + off;

        xlib::XSetForeground(x.dpy, x.gc, if off == 0 {
            0x80000000
        } else {
            0x80FFFFFF
        });

        while _y < (oy + h) {
            xlib::XDrawPoint(x.dpy, x.window, x.gc, ox, _y);
            _y += 2;
        }
    }
}

unsafe fn draw_dotted_line_horiz_full(x: &X, area: &Area, yoff: i32) {
    draw_dotted_line_horiz(x, area.x, area.y + yoff, area.w);
}

unsafe fn draw_dotted_line_vert_full(x: &X, area: &Area, xoff: i32) {
    draw_dotted_line_vert(x, area.x + xoff, area.y, area.h);
}

unsafe fn drawdottedcross(x: &X, area: &Area, rx: i32, ry: i32) {
    draw_dotted_line_horiz(x, area.x + rx - area.w / 12, area.y + ry, area.w / 6);
    draw_dotted_line_vert(x, area.x + rx, area.y + ry - area.h / 12, area.h / 6);
}

fn draw(x: &X, area: &Area) {
    unsafe {
        xlib::XClearWindow(x.dpy, x.window);

        draw_dotted_line_horiz_full(x, area, -1);
        draw_dotted_line_horiz_full(x, area, area.h / 3);
        draw_dotted_line_horiz_full(x, area, 2 * area.h / 3);
        draw_dotted_line_horiz_full(x, area, area.h);

        for rx in 0..3 {
            for ry in 0..3 {
                drawdottedcross(x, area, (1 + rx * 2) * area.w / 6, (1 + ry * 2) * area.h / 6);
            }
        }

        draw_dotted_line_vert_full(x, area, -1);
        draw_dotted_line_vert_full(x, area, area.w / 3);
        draw_dotted_line_vert_full(x, area, 2 * area.w / 3);
        draw_dotted_line_vert_full(x, area, area.w);

        xlib::XFlush(x.dpy);
    }
}

fn get_key_press(x: &X) -> Option<(u32, u32)> {
    unsafe {
        let mut ev: xlib::XEvent = MaybeUninit::zeroed().assume_init();

        loop {
            if xlib::XNextEvent(x.dpy, &mut ev) != 0 { return None; }

            if ev.type_ == xlib::KeyPress { return Some((ev.key.keycode, ev.key.state)); }

            xlib::XRaiseWindow(x.dpy, x.window);
        }
    }
}

fn move_cursor_and_click(x: &X, button: i32, ox: i32, oy: i32, no_release: bool) {
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

fn move_cursor_edge_and_click(x: &X, area: &Area, segment: u32, button: i32, no_release: bool) {
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

fn close_x(x: X) {
    unsafe {
        xlib::XUnmapWindow(x.dpy, x.window);
        xlib::XDestroyWindow(x.dpy, x.window);
        xlib::XFreeGC(x.dpy, x.gc);
        xlib::XFreeColormap(x.dpy, x.colormap);
        xlib::XCloseDisplay(x.dpy);
    }
    
    drop(x);
}

fn third(pos: i32, size: i32, segment: u32) -> (i32, i32) {
    match segment {
        0 => (pos, size / 3),
        1 => (pos + size / 3 + 1, 2 * size / 3 - size / 3 - 1),
        2 => (pos + 2 * size / 3 + 1, size - 2 * size / 3 - 1),
        _ => (pos, size)
    }
}

fn run(x: &X) {
    let mut area = Area {
        x: 0,
        y: 0,
        w: x.w,
        h: x.h
    };

    let mut history: Vec<Area> = Vec::new();

    loop {
        draw(&x, &area);

        let (key, state) = match get_key_press(&x) {
            Some(val) => val,
            None => break
        };

        println!("key: {} {}", key, state);

        if key == 66 {
            if state != 0 { break }

            area = match history.pop() {
                Some(val) => val,
                None => break 
            };
        }

        let shift: bool = state & 1 == 1;

        if key >= 24 && key <= 32 { move_cursor_edge_and_click(x, &area, key - 24, 1, shift); }
        if key >= 10 && key <= 18 { move_cursor_edge_and_click(x, &area, key - 10, 2, shift); }
        if key >= 52 && key <= 60 { move_cursor_edge_and_click(x, &area, key - 52, 3, shift); }

        if key == 20 { move_cursor_and_click(x, 5, -1, -1, false); }
        if key == 21 { move_cursor_and_click(x, 4, -1, -1, false); }

        if key >= 38 && key <= 46 {
            let i = key - 38;

            history.push(area.clone());

            (area.x, area.w) = third(area.x, area.w, i % 3);
            (area.y, area.h) = third(area.y, area.h, i / 3);
        }
    }
}

fn main() {
    /*
     * TODO LIST
     *
     * - just move the cursor without clicking
     * - some sort of config file for custom bindings
     * - test what happens with multiple screens
     */

    let x = match start_x() {
        Some(x) => x,
        None => return
    };

    run(&x);

    close_x(x);
}

