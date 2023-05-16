# 利用Rust编写CHIP8模拟器

```
USAGE:
    yet-another-rchip8.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -r, --rom <ROM>    Sets the rom file to load
```

Requirements 
- [SDL2](https://github.com/Rust-SDL2/rust-sdl2#requirements)

macOS
```
brew install sdl2 sdl2_mixer sdl2_image sdl2_ttf sdl2_gfx
```

example:

```
cargo run -- --rom .\test\TETRIS
```

## 部分参考内容：

- [wikipedia CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [Rust-SDL2](https://github.com/Rust-SDL2/rust-sdl2)
- [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator)
- [CHIP-8 Archive](https://johnearnest.github.io/chip8Archive/)
- [chip8-test-rom](https://github.com/corax89/chip8-test-rom)
- [fonts](https://github.com/mattmikolay/chip-8/issues/3)
- [mattmikolay/chip-8](https://github.com/mattmikolay/chip-8)
- [Octo](https://internet-janitor.itch.io/octo)
- [David Winter's CHIP-8 emulation page](http://www.pong-story.com/chip8/)
