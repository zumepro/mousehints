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

struct Area {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

unsafe fn drawdottedlinehoriz(x: &X, area: &Area, yoff: i32, off: bool) {
    let mut _x = area.x + if off { 1 } else { 0 };
    let _y = area.y + yoff;

    while _x < (area.x + area.w) {
        xlib::XDrawPoint(x.dpy, x.window, x.gc, _x, _y);
        _x += 2;
    }
}

unsafe fn drawdottedlinevert(x: &X, area: &Area, xoff: i32, off: bool) {
    let _x = area.x + xoff;
    let mut _y = area.y + if off { 1 } else { 0 };

    while _y < (area.y + area.h) {
        xlib::XDrawPoint(x.dpy, x.window, x.gc, _x, _y);
        _y += 2;
    }
}

fn draw(x: &X, area: &Area) {
    unsafe {
        xlib::XClearWindow(x.dpy, x.window);

        xlib::XSetForeground(x.dpy, x.gc, 0x80000000);

        drawdottedlinehoriz(x, area, -1, false);
        drawdottedlinehoriz(x, area, area.h / 3, false);
        drawdottedlinehoriz(x, area, 2 * area.h / 3, false);
        drawdottedlinehoriz(x, area, area.h, false);

        drawdottedlinevert(x, area, -1, false);
        drawdottedlinevert(x, area, area.w / 3, false);
        drawdottedlinevert(x, area, 2 * area.w / 3, false);
        drawdottedlinevert(x, area, area.w, false);

        xlib::XSetForeground(x.dpy, x.gc, 0x80FFFFFF);

        drawdottedlinehoriz(x, area, -1, true);
        drawdottedlinehoriz(x, area, area.h / 3, true);
        drawdottedlinehoriz(x, area, 2 * area.h / 3, true);
        drawdottedlinehoriz(x, area, area.h, true);

        drawdottedlinevert(x, area, -1, true);
        drawdottedlinevert(x, area, area.w / 3, true);
        drawdottedlinevert(x, area, 2 * area.w / 3, true);
        drawdottedlinevert(x, area, area.w, true);

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

fn move_cursor_and_close(x: X, button: u32, ox: i32, oy: i32) {
    unsafe {
        xlib::XUnmapWindow(x.dpy, x.window);

        if ox >= 0 && oy >= 0 {
            xlib::XWarpPointer(x.dpy, 0, x.rootwin, 0, 0, 0, 0, ox, oy);
        }

        xtest::XTestFakeButtonEvent(x.dpy, button, 1, 0);
        xlib::XSync(x.dpy, 0);
        xtest::XTestFakeButtonEvent(x.dpy, button, 0, 0);
        xlib::XSync(x.dpy, 0);
        
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

fn edges(area: &Area, segment: u32, button: u32) -> (i32, i32, u32) {
    let x = match segment % 3 {
        0 => area.x,
        1 => area.x + area.w / 2,
        2 => area.x + area.w - 1,
        _ => 0  
    };
    
    let y = match segment / 3 {
        0 => area.y,
        1 => area.y + area.h / 2,
        2 => area.y + area.h - 1,
        _ => 0  
    };

    (x, y, button)
}

fn run() -> bool {
    let x = match start_x() {
        Some(x) => x,
        None => return false
    };

    let mut area = Area {
        x: 0,
        y: 0,
        w: x.w,
        h: x.h
    };

    let (ox, oy, button): (i32, i32, u32) = loop {
        draw(&x, &area);

        let (key, state) = match get_key_press(&x) {
            Some(val) => val,
            None => break (0, 0, 0)
        };

        println!("key: {} {}", key, state);

        if key == 66 { break (0, 0, 0); }

        if key >= 24 && key <= 32 { break edges(&area, key - 24, 1); }
        if key >= 10 && key <= 18 { break edges(&area, key - 10, 2); }
        if key >= 52 && key <= 60 { break edges(&area, key - 52, 3); }

        if key == 20 { break (-1, -1, 4); }
        if key == 21 { break (-1, -1, 5); }

        if key >= 38 && key <= 46 {
            // TODO - use some sort of stack for undo

            let i = key - 38;

            (area.x, area.w) = third(area.x, area.w, i % 3);
            (area.y, area.h) = third(area.y, area.h, i / 3);
        }
    };

    if button == 0 {
        return false;
    }

    // TODO - do not actually restart everything (maybe just hide the window?)
    move_cursor_and_close(x, button, ox, oy);

    true
}

fn main() {
    // TODO - a config file

    loop {
        if !run() { break; }
    }
}

