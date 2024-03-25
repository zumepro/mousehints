use x11::xlib;
use std::mem::MaybeUninit;

pub struct X {
    pub dpy: *mut xlib::Display,
    pub rootwin: xlib::Window,
    pub window: xlib::Window,
    pub colormap: xlib::Colormap,
    pub gc: xlib::GC,
    pub w: i32,
    pub h: i32
}

#[derive(Clone)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32
}

pub fn start_x() -> Option<X> {
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

pub fn close_x(x: X) {
    unsafe {
        xlib::XUnmapWindow(x.dpy, x.window);
        xlib::XDestroyWindow(x.dpy, x.window);
        xlib::XFreeGC(x.dpy, x.gc);
        xlib::XFreeColormap(x.dpy, x.colormap);
        xlib::XCloseDisplay(x.dpy);
    }
    
    drop(x);
}

