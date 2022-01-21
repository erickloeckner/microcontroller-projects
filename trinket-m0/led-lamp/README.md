# Linux build directions


### Compile project as binary:
```
cargo objcopy --release --features unproven -- -O binary out.bin
```

### Convert binary into UF2:
```
uf2conv out.bin --base 0x2000 --output out.uf2
```

### Double press the reset button on Trinket M0 to enter bootloader mode, then drag UF2 file into the drive that appears named TRINKETBOOT
