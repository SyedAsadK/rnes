# rnes

**rnes** is a Rust-based NES (Nintendo Entertainment System) emulator.  
This project is a personal attempt to learn about emulator development, the 6502 CPU, and NES graphics rendering using modern Rust.

![Demo Screenshot](demo_screenshot.png)

## Features

- CPU: Partial 6502 emulation
- Graphics: Basic 32x32 framebuffer rendering (SDL3)
- Input: Keyboard controls (WASD for movement, Escape to quit)
- Game Example: Includes a simple "snake" game ROM
- Random number support for in-game events

## Getting Started

### Prerequisites

- Rust (latest stable recommended)
- [SDL3](https://github.com/libsdl-org/SDL) development libraries (ensure they are installed on your system)

### Building

Clone the repository:

```bash
git clone https://github.com/SyedAsadK/rnes.git
cd rnes
```

Build and run:

```bash
cargo run --release
```

> **Note:** If you get build errors related to SDL, make sure you have the SDL3 development libraries installed.

### Controls

- **W**: Up
- **A**: Left
- **S**: Down
- **D**: Right
- **Escape**: Quit

## Project Structure

- `src/comp/cpu.rs` — 6502 CPU emulation core
- `src/main.rs` — SDL setup, emulation loop, game integration
- `src/comp/` — Additional components (future expansion)
- `resources/` — (Optional) Place for ROMs or assets

## Known Issues / Limitations

- Not a full-featured NES emulator (yet)
- Only supports basic framebuffer output (32x32 grid)
- Only runs bundled simple games for now

## Contributing

Pull requests are welcome! Feel free to suggest improvements, submit bug fixes, or help expand emulator functionality.

## License

MIT License

---

*This project was inspired by [OneLoneCoder's NES emulator series](https://youtube.com/playlist?list=PLrOv9FMX8xJE8NgepZR1etrsU63fDDGxO) and Rust emulation tutorials.*
