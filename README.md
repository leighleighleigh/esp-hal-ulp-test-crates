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
