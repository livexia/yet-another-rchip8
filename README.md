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

example:
```
$env:RUST_LOG="debug"; cargo run -- --rom .\test\TETRIS
```