# Build Directions

Compile binary:
```
cargo objcopy --release --features unproven -- -O binary out.bin
```

Convert to UF2:
```
uf2conv out.bin --base 0x2000 --output out.uf2
```
