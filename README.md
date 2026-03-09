# Keymouse

Control your mouse on macOS with fast, Vim-style keyboard navigation.

![Keymouse Demo](demo.gif)

*Demo: toggling mouse mode, moving with `H J K L`, clicking, and jumping the cursor with the grid system.*

## Features

- Move the cursor with **H J K L**
- Trigger mouse clicks from the keyboard (`F` for left click, `D` for right click)
- Jump the cursor with a 3x3 grid-based targeting system
- Multi-monitor aware cursor jumping
- System-level keyboard interception using macOS `CGEventTap`
- Built in **Rust** with low-level event handling

## How It Works

- Press **F8** to toggle mouse mode on or off.
- In mouse mode, use **H J K L** to move the cursor.
- Press **;** to open jump grid mode.
- Press one grid key (`QWE / ASD / ZXC`) to instantly move the cursor.

## Installation

Clone the repository:

```bash
git clone https://github.com/debacodes10/keymouse.git
cd keymouse
```

Build a release binary:

```bash
cargo build --release
```

The compiled binary will be available at:

```bash
target/release/keymouse
```

## Usage

Start the tool:

```bash
cargo run --release
```

Grant macOS permissions to the app launching Keymouse (usually your terminal):

- `System Settings` -> `Privacy & Security` -> `Accessibility`
- `System Settings` -> `Privacy & Security` -> `Input Monitoring`

| Key     | Action            |
| ------- | ----------------- |
| F8      | Toggle mouse mode |
| H J K L | Move cursor       |
| F       | Left click        |
| D       | Right click       |
| ;       | Open jump grid    |

## Configuration

Keymouse loads configuration from:

```text
~/.config/keymouse/config.toml
```

If the file is missing, Keymouse uses built-in defaults. At startup, it also writes an example file to that path so you can customize bindings.

Example `config.toml`:

```toml
movement_up = "k"
movement_down = "j"
movement_left = "h"
movement_right = "l"

grid_key = ";"
confirm_key = "enter"

left_click = "f"
right_click = "d"

fast_modifier = "shift"
slow_modifier = "option"
```

## Roadmap

- [x] Vim-style cursor movement
- [x] Grid jump navigation
- [x] Multi-monitor support
- [ ] Recursive grid zoom
- [x] Custom key bindings
- [x] Configuration file
- [ ] Homebrew installation

## Contributing

Contributions, suggestions, and feature requests are welcome. Open an issue to discuss ideas, or submit a pull request with a focused change.

## License

This repository does not currently include a license file.
