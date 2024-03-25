mod xcommon;
mod draw;
mod io;
use xcommon::Area;
use xcommon::X;

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
        draw::draw(&x, &area);

        let (key, state) = match io::get_key_press(&x) {
            Some(val) => val,
            None => break
        };

        println!("key: {} {}", key, state);

        let shift: bool = state & 1 == 1;

        if key == 66 {
            if shift { break }

            area = match history.pop() {
                Some(val) => val,
                None => break 
            };
        }

        if key >= 24 && key <= 32 { io::move_cursor_edge_and_click(x, &area, key - 24, 1, shift); }
        if key >= 10 && key <= 18 { io::move_cursor_edge_and_click(x, &area, key - 10, 2, shift); }
        if key >= 52 && key <= 60 { io::move_cursor_edge_and_click(x, &area, key - 52, 3, shift); }

        if key == 20 { io::move_cursor_and_click(x, 5, -1, -1, false); }
        if key == 21 { io::move_cursor_and_click(x, 4, -1, -1, false); }

        if key == 65 {
            loop {
                area = match history.pop() {
                    Some(val) => val,
                    None => break
                }
            }
        }

        if key >= 38 && key <= 46 {
            if shift {
                io::move_cursor_edge_and_click(x, &area, key - 38, -1, false);
                continue
            }

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
     * - some sort of config file for custom bindings
     * - test what happens with multiple screens
     */

    let x = match xcommon::start_x() {
        Some(x) => x,
        None => return
    };

    run(&x);

    xcommon::close_x(x);
}

