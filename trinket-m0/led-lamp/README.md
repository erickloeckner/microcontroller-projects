// Linux build directions
//
// Move into the project directory:
cd led-lamp

// Compile project as binary:
cargo objcopy --release --features unproven -- -O binary out.bin

// Convert binary into UF2. Most ATSAMD boards need the --base 0x2000 flag:
uf2conv out.bin --base 0x2000 --output out.uf2
