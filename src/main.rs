use evdev::{Device, EventType, InputEventKind};
use minifb::{Key, Window, WindowOptions};
use std::{
    thread,
    time::{Duration, Instant},
};

const W: usize = 1000;
const H: usize = 700;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: sudo ./mouse_visualizer /dev/input/eventX");

    let mut device = Device::open(path).expect("Cannot open mouse");

    println!("Device: {:?}", device.name());

    let mut window =
        Window::new("Raw Mouse Event Visualizer", W, H, WindowOptions::default()).unwrap();

    let mut buffer = vec![0xFFFFFFu32; W * H];

    let (mut x, mut y) = (W as i32 / 2, H as i32 / 2);

    let mut count = 0u64;
    let mut last = Instant::now();
    let mut rate = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if event.event_type() == EventType::RELATIVE {
                        match event.kind() {
                            InputEventKind::RelX => x += event.value(),
                            InputEventKind::RelY => y += event.value(),
                            _ => continue,
                        }

                        // Count one report when an X/Y movement arrives.
                        // REL_X and REL_Y are separate events, so count
                        // only REL_Y to avoid double-counting.
                        if matches!(event.kind(), InputEventKind::RelY) {
                            count += 1;

                            if x >= 0 && x < W as i32 && y >= 0 && y < H as i32 {
                                buffer[y as usize * W + x as usize] = 0x000000;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Read error: {e}");
                break;
            }
        }

        if last.elapsed() >= Duration::from_secs(1) {
            rate = count as f64 / last.elapsed().as_secs_f64();
            println!("Events/sec: {:.0}", rate);
            count = 0;
            last = Instant::now();
        }

        window.set_title(&format!("Mouse events/sec: {:.0}", rate));

        window.update_with_buffer(&buffer, W, H).unwrap();

        thread::sleep(Duration::from_millis(1));
    }
}
