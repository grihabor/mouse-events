use evdev::{Device, EventSummary, RelativeAxisCode};
use minifb::{Key, Window, WindowOptions};
use std::{
    thread,
    time::{Duration, Instant},
};

const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 50.0;
const ZOOM_STEP: f64 = 1.1;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: sudo ./mouse_visualizer /dev/input/eventX");

    let mut device = Device::open(path).expect("Cannot open mouse");

    println!("Device: {:?}", device.name());

    // Size the window/buffer to the primary display.
    let (w, h) = primary_screen_size();
    println!("Screen size: {w}x{h}");

    let mut window = Window::new(
        "Raw Mouse Event Visualizer",
        w,
        h,
        WindowOptions {
            borderless: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut buffer = vec![0xFFFFFFu32; w * h];

    // Current pen position in world (raw-event) coordinates.
    let (mut x, mut y) = (w as i32 / 2, h as i32 / 2);

    // The camera is fixed at the start position; zoom is centered on it.
    let cam_x = w as f64 / 2.0;
    let cam_y = h as f64 / 2.0;
    let mut zoom = 1.0f64;

    // The drawn line, stored in world coordinates so it can be
    // re-rendered at any zoom level.
    let mut points: Vec<(i32, i32)> = Vec::new();

    let mut count = 0u64;
    let mut last = Instant::now();
    let mut rate = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    match event.destructure() {
                        EventSummary::RelativeAxis(_, RelativeAxisCode::REL_X, value) => {
                            x += value;
                        }
                        EventSummary::RelativeAxis(_, RelativeAxisCode::REL_Y, value) => {
                            y += value;

                            // Count one report when an X/Y movement arrives.
                            // REL_X and REL_Y are separate events, so count
                            // only REL_Y to avoid double-counting.
                            count += 1;

                            points.push((x, y));
                        }
                        // Mouse wheel zooms the view.
                        EventSummary::RelativeAxis(_, RelativeAxisCode::REL_WHEEL, value) => {
                            zoom *= ZOOM_STEP.powi(value);
                        }
                        _ => continue,
                    }
                }
            }
            Err(e) => {
                eprintln!("Read error: {e}");
                break;
            }
        }

        // Keyboard zoom controls as an alternative to the wheel.
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) {
            zoom *= ZOOM_STEP;
        }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) {
            zoom /= ZOOM_STEP;
        }
        zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        // Re-render the whole trail at the current zoom.
        buffer.iter_mut().for_each(|p| *p = 0xFFFFFF);
        let block = zoom.ceil() as i32;
        for &(wx, wy) in &points {
            let sx = ((wx as f64 - cam_x) * zoom + w as f64 / 2.0).floor() as i32;
            let sy = ((wy as f64 - cam_y) * zoom + h as f64 / 2.0).floor() as i32;
            draw_block(&mut buffer, w, h, sx, sy, block);
        }

        if last.elapsed() >= Duration::from_secs(1) {
            rate = count as f64 / last.elapsed().as_secs_f64();
            println!("Events/sec: {:.0}", rate);
            count = 0;
            last = Instant::now();
        }

        window.set_title(&format!("Mouse events/sec: {:.0}  |  zoom: {:.2}x", rate, zoom));

        window.update_with_buffer(&buffer, w, h).unwrap();

        thread::sleep(Duration::from_millis(1));
    }
}

/// Resolution of the default X11 screen, falling back to 1920x1080 if it
/// cannot be queried. Uses `x11-dl`, which loads libX11 at runtime (no
/// link-time dependency), and works under XWayland too.
fn primary_screen_size() -> (usize, usize) {
    unsafe {
        let Ok(xlib) = x11_dl::xlib::Xlib::open() else {
            return (1920, 1080);
        };
        let display = (xlib.XOpenDisplay)(std::ptr::null());
        if display.is_null() {
            return (1920, 1080);
        }
        let screen = (xlib.XDefaultScreen)(display);
        let w = (xlib.XDisplayWidth)(display, screen);
        let h = (xlib.XDisplayHeight)(display, screen);
        (xlib.XCloseDisplay)(display);

        if w > 0 && h > 0 {
            (w as usize, h as usize)
        } else {
            (1920, 1080)
        }
    }
}

/// Draw a filled `size`x`size` block with top-left at (`sx`, `sy`),
/// clipped to the buffer bounds.
fn draw_block(buffer: &mut [u32], w: usize, h: usize, sx: i32, sy: i32, size: i32) {
    let size = size.max(1);
    for dy in 0..size {
        for dx in 0..size {
            let px = sx + dx;
            let py = sy + dy;
            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                buffer[py as usize * w + px as usize] = 0x000000;
            }
        }
    }
}
