# esp-hal-ulp-test-crates

I always end up building the same 'testing project structure', using `shell.nix` and `justfile`-s,
so I've finally gotten around to making a repo from it.

It currently has two crates within it...
## hp-blinky
A multi-target crate, which uses feature-flags to build for `esp32s3`, `esp32s2`, or `esp32c6`.
Points to my local `esp-hal` fork by default.
Depends on the binary built from `ulp-blinky`, which is copied into `./ulp-apps/` folder by the `justfile`.

## ulp-blinky
A simple ULP program for `esp32s3/esp32s3 RISCV ULP`, and `esp32c6 LP` cores.
It starts up, increments a counter at a fixed memory address, and then shuts down.
(This crate requires my fork of `esp-hal`, which implements `RTC_TIMER1` by allowing `main` to exit.)

# Basic Usage

0. Install NixOS/Nix, so that you have `nix-shell` available.
1. Clone this repo
2. Open a terminal in the `hp-blinky` directory
3. Run `nix-shell` to enter the development environment, which uses [esp-rs-nix](https://github.com/leighleighleigh/esp-rs-nix).
4. Plug in your `esp32s3` in development mode.
5. Inspect the `justfile`, make sure you are happy with it. Visit [just.systems](https://just.systems/) to learn more.
6. Run `just`, which will...
 - Build the `ulp-blinky` crate, copy the resulting binary into the `hp-blinky` crate.
 - Build the `hp-blinky` crate
 - Flash the `hp-blinky` crate
 - Open a serial monitor

The output should be similar to the following...

```shell
(esp-hal-ulp-tests)leigh@leigh-desktop:hp-blinky$ just 
    Finished `release` profile [optimized] target(s) in 0.08s
'ulp-blinky/target/riscv32imc-unknown-none-elf/release/blinky' -> 'hp-blinky/ulp-apps/esp32s3-ulp-blinky'
   Compiling hp-blinky v0.1.0 (/home/leigh/Git/esp-hal-ulp-test-crates/hp-blinky)
    Finished `release` profile [optimized] target(s) in 0.88s
[2026-03-07T01:06:14Z INFO ] 🚀 A new version of espflash is available: v4.3.0
[2026-03-07T01:06:14Z WARN ] Monitor options were provided, but `--monitor/-M` flag isn't set. These options will be ignored.
[2026-03-07T01:06:14Z INFO ] Serial port: '/dev/ttyACM0'
[2026-03-07T01:06:14Z INFO ] Connecting...
[2026-03-07T01:06:15Z INFO ] Using flash stub
Chip type:         esp32s3 (revision v0.2)
Crystal frequency: 40 MHz
Flash size:        4MB
Features:          WiFi, BLE, Embedded Flash
MAC address:       24:ec:4a:31:80:1c
App/part. size:    118,720/2,097,152 bytes, 5.66%
 [========================================]      14/14      0x0      Verifying... OK!
 [========================================]       1/1       0x8000   Verifying... OK!
 [========================================]      44/44      0x10000  Verifying... OK!
Flashing has completed!
# [1772845577] Connected to /dev/ttyACM0
18446744073709551615
I (123) esp_image: segment 4: paddr=00020020 vaddr=42010020 size=0cf78h ( 53112) map
I (138) boot: Loaded app from partition at offset 0x10000
I (138) boot: Disabling RNG early entropy source...
INFO - Rolling count 1, delta 99ms, rate 10.1010101010101Hz
INFO - Rolling count 2, delta 197ms, rate 10.152284263959391Hz
INFO - Rolling count 3, delta 296ms, rate 10.135135135135135Hz
INFO - Rolling count 4, delta 395ms, rate 10.126582278481012Hz
INFO - Rolling count 5, delta 494ms, rate 10.121457489878543Hz
INFO - Rolling count 6, delta 593ms, rate 10.118043844856661Hz
INFO - Rolling count 7, delta 692ms, rate 10.115606936416185Hz
INFO - Rolling count 8, delta 791ms, rate 10.11378002528445Hz
INFO - Rolling count 9, delta 889ms, rate 10.123734533183352Hz
INFO - Rolling count 10, delta 988ms, rate 10.121457489878543Hz
INFO - Rolling count 11, delta 1087ms, rate 10.119595216191353Hz
INFO - Rolling count 12, delta 1186ms, rate 10.118043844856661Hz
INFO - Rolling count 13, delta 1285ms, rate 10.116731517509729Hz
INFO - Rolling count 14, delta 1384ms, rate 10.115606936416185Hz
```