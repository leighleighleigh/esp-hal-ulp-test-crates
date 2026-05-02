###### LINTING ######

# chip - The ESP32 chip to target, for both HP and LP crates.
chip := "esp32s3" 
target := "xtensa-esp32s3-none-elf"
#esp32c6 = "build --release --features=esp32c6 --target riscv32imac-unknown-none-elf"
#esp32s2 = "build --release --features=esp32s2 --target xtensa-esp32s2-none-elf"
#esp32s3 = "build --release --features=esp32s3 --target xtensa-esp32s3-none-elf"

format:
  #!/usr/bin/env bash
  shopt -s globstar 
  cargo fmt --all -- --config-path=rustfmt.toml src/**/*.rs

clippy *CLIPPY_ARGS=("--features " + chip + " --target " + target):
  #!/usr/bin/env bash
  cargo clippy --release {{CLIPPY_ARGS}} -- -D warnings --no-deps
