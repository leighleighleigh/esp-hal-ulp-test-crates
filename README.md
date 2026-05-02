# esp-hal-ulp-test-crates

I always end up building the same 'testing project structure', using `shell.nix` and `justfile`-s,
so I've finally gotten around to making a repo from it.

# Repo structure

```shell
(esp-hal-ulp-tests)leigh@leigh-desktop:esp-hal-ulp-test-crates$ tree -L 2
.
├── counter              # A simple counting demo.
│   ├── hp-blinky
│   └── ulp-blinky
├── interrupts           # WIP
│   ├── hp-gpio-wakeup
│   └── ulp-baremetal
├── hp_crate.just
├── binutils.just
├── clippy.just
├── openocd.cfg
├── README.md
├── rustfmt.toml
├── rust-toolchain.toml
└── shell.nix                             
```

# Basic Usage

0. Install NixOS/Nix, so that you have `nix-shell` available.
1. Clone this repo
2. Open a terminal in the `counter/hp-blinky` directory
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
just all # build, flash, monitor
    Finished `release` profile [optimized] target(s) in 0.08s
   Compiling hp-blinky v0.1.0 (/home/leigh/Git/esp-hal-ulp-test-crates/counter/hp-blinky)
    Finished `release` profile [optimized] target(s) in 0.71s
...
# [1777687716] Connected to /dev/ttyACM0
18446744073709551615
I (124) esp_image: segment 4: paddr=00020020 vaddr=42010020 size=08b90h ( 35728) map
I (134) boot: Loaded app from partition at offset 0x10000
I (134) boot: Disabling RNG early entropy source...
INFO - LP core will be woken from HP Cpu
INFO - [+112 us] counter: 0
INFO - [+973084 us] counter: 1
INFO - [+973297 us] counter: 2
INFO - [+973386 us] counter: 3
INFO - [+973332 us] counter: 4
INFO - [+973407 us] counter: 5
INFO - [+973372 us] counter: 6
```
