use x11::xlib;
use crate::xcommon::X;
use crate::xcommon::Area;

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

pub fn draw(x: &X, area: &Area) {
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
