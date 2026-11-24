# mouse-events

A raw mouse event visualizer. It reads relative movement events directly from a
Linux evdev input device and draws the accumulated path in a borderless,
screen-sized window.

## Build

```sh
cargo build --release
```

## Run

Find your mouse device under `/dev/input/eventX` (e.g. via `evtest` or
`libinput list-devices`), then run with access to that device:

```sh
sudo ./target/release/mouse-events /dev/input/eventX
```

Reading evdev devices usually requires root (or membership in the `input`
group).

## Controls

| Input                   | Action                         |
| ----------------------- | ------------------------------ |
| Mouse wheel             | Zoom in / out (anchored at the cursor) |
| `+` / `-` (and numpad)  | Zoom in / out                  |
| `Esc`                   | Quit                           |

The window title shows the current event rate (events/sec) and zoom level.

## Notes

- Movement is accumulated from raw `REL_X` / `REL_Y` events with no pointer
  acceleration, so the drawn path does not track the OS cursor.
- Screen size is queried via X11 (`x11-dl`, loaded at runtime), which also works
  under XWayland. It falls back to 1920x1080 if it cannot be determined.
