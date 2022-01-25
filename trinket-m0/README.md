# Adafruit Trinket M0

![Adafruit Trinket M0 photo](https://cdn-shop.adafruit.com/145x109/3500-04.jpg)

Product page: https://www.adafruit.com/product/3500

## Toolchain Requirements:

(Linux)
```
$ rustup component add llvm-tools-preview
$ cargo install uf2conv cargo-binutils
```

Projects use the ATSAMD HAL and PAC crates. More information is here: https://github.com/atsamd-rs/atsamd
