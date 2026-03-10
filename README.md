# Keymouse

Control your mouse on macOS with fast, Vim-style keyboard navigation.

![Keymouse Demo](demo.gif)

*Demo: toggling mouse mode, moving with `H J K L`, clicking, and jumping the cursor with the grid system.*

## Features

- Move the cursor with **H J K L** (normal speed)
- Hold **Shift** for fast movement, **Option/Alt** for slow movement
- Trigger mouse clicks from the keyboard (`F` for left click, `D` for right click)
- Jump the cursor with a recursive 3x3 grid targeting system
- Translucent on-screen grid overlay with cell labels (`Q W E / A S D / Z X C`)
- Live overlay depth indicator while zooming (`Depth: 0`, `Depth: 1`, ...)
- Multi-monitor aware cursor jumping
- System-level keyboard interception using macOS `CGEventTap`
- Built in **Rust** with low-level event handling

## How It Works

- Press **F8** to toggle mouse mode on or off.
- In mouse mode, use **H J K L** to move the cursor.
- Hold **Shift** while moving for fast steps, or **Option/Alt** for slow steps.
- Press **F** for left click, **D** for right click.
- Press **;** to open jump grid mode (overlay appears on the active display).
- Press grid keys (`QWE / ASD / ZXC`) to zoom recursively into cells.
- Press **Enter** to confirm jump to the center of the selected cell.
- Press **Esc** to cancel grid mode.

## Installation

### Option 1: Install from crates.io (recommended)

```bash
cargo install keymouse
```

Run it:

```bash
keymouse
```

### Option 2: Download a prebuilt binary from GitHub Releases

Latest release:

- https://github.com/debacodes10/keymouse/releases/latest

Example (Apple Silicon / arm64):

```bash
curl -L -o keymouse-macos-arm64.zip https://github.com/debacodes10/keymouse/releases/latest/download/keymouse-macos-arm64.zip
unzip keymouse-macos-arm64.zip
chmod +x keymouse-macos-arm64
./keymouse-macos-arm64
```

### Option 3: Build from source

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
keymouse
```

If you built from source, run:

```bash
./target/release/keymouse
```

Grant macOS permissions to the app launching Keymouse (usually your terminal):

- `System Settings` -> `Privacy & Security` -> `Accessibility`
- `System Settings` -> `Privacy & Security` -> `Input Monitoring`

| Key                | Action                                                          |
| ------------------ | --------------------------------------------------------------- |
| F8                 | Toggle mouse mode                                               |
| H J K L            | Move cursor (normal speed)                                      |
| Shift + H/J/K/L    | Move cursor fast                                                |
| Option + H/J/K/L   | Move cursor slow                                                |
| F                  | Left click                                                      |
| D                  | Right click                                                     |
| ;                  | Open jump grid overlay on active display                        |
| Q/W/E/A/S/D/Z/X/C  | Select grid cell and keep zooming in                            |
| Enter              | Confirm grid selection and move cursor to selected cell center  |
| Esc                | Cancel grid mode                                                |

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
- [x] Recursive grid zoom
- [x] Custom key bindings
- [x] Configuration file
- [ ] Homebrew installation

## Contributing

Contributions, suggestions, and feature requests are welcome. Open an issue to discuss ideas, or submit a pull request with a focused change.

## License

This repository does not currently include a license file.
