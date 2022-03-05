

cargo objcopy --release --features unproven -- -O binary out.bin && uf2conv out.bin --base 0x4000 --output out.uf2
