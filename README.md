# keymouse

`keymouse` is a macOS developer productivity tool that lets you control the mouse from the keyboard (Vim-style), with true key interception via `CGEventTap`.

## Features

- Global keyboard interception on macOS using Quartz `CGEventTap`
- Mouse mode toggle with `F8`
- Relative mouse movement with `H`, `J`, `K`, `L`
- Mouse clicks with keyboard:
  - `F` = left click
  - `D` = right click
- Cursor Jump Grid (3x3) with two keystrokes:
  - `;` enters grid mode
  - `Q W E / A S D / Z X C` jumps to a grid region center
- Multi-monitor aware grid jumps (uses the display containing the current cursor)
- Event suppression while mouse/grid mode is active (keys do not leak to apps)

## Keymap

### Normal mode

- Keyboard behaves normally
- Events pass through to apps

### Mouse mode (`F8`)

- `H` -> move left
- `J` -> move down
- `K` -> move up
- `L` -> move right
- `F` -> left click
- `D` -> right click

While mouse mode is active, keyboard events are intercepted and suppressed by the event tap.

### Grid mode (`;` while mouse mode is ON)

Press one of:

```text
Q W E
A S D
Z X C
```

Mapping:

- `Q` top-left
- `W` top-center
- `E` top-right
- `A` middle-left
- `S` center
- `D` middle-right
- `Z` bottom-left
- `X` bottom-center
- `C` bottom-right

After a valid grid jump, grid mode exits automatically and returns to mouse mode.

## Requirements

- macOS
- Rust toolchain (`cargo`, `rustc`)

## Build and run

```bash
cargo run
```

You should see a startup message in stderr. Press `F8` to toggle mouse mode.

## macOS permissions

For global keyboard interception and mouse control, grant permissions to the app that launches `keymouse` (usually your terminal, or the built binary):

1. Open `System Settings` -> `Privacy & Security`.
2. In `Accessibility`, add/enable your terminal (or `keymouse` binary).
3. In `Input Monitoring`, add/enable your terminal (or `keymouse` binary).
4. Restart the terminal/app after granting permissions.

Without these permissions, event tap creation or behavior may fail.

## Implementation notes

- Input interception is implemented with `CGEventTapCreate` at `kCGHIDEventTap` (`HeadInsertEventTap`).
- Key suppression is done by returning `NULL` from the callback for intercepted events.
- Mouse actions are performed through `enigo`.
- No polling loop and no backspace hacks.

## Dependencies

- `core-graphics`
- `core-foundation`
- `enigo`
