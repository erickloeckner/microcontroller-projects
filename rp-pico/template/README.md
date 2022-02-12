# Build Directions

- navigate to project directory and compile project to ELF:
```
cargo build --release
```

- convert ELF to UF2:
```
elf2uf2-rs ./target/thumbv6m-none-eabi/release/template template.uf2
```

- hold reset button while plugging in the board, then drag UF2 into the drive that appears


# Alternate Method
- the binary can be compiled and uploaded directly if the board is already in bootloader mode:
```
cargo run --release
```
