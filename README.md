# esp-hal-ulp-test-crates

I've been slowly porting the `esp-idf` [ULP RISC-V test suite](https://github.com/espressif/esp-idf/blob/master/components/ulp/test_apps/ulp_riscv/main/test_ulp_riscv.c) to Rust, 
while I work on adding new features to `esp-lp-hal`.  

```shell
.
├── hp-app              # Main crate containing the HP-LP integration tests.
├── lp-app              # Companion crate, runs on the ULP/LP core, responds to test commands.
├── shared              # Provides shared variable support between the HP/LP cores. Undocumented.
├── binutils.just       
├── clippy.just
├── README.md           # You are here
├── rustfmt.toml
├── rust-toolchain.toml
└── shell.nix
```

# Feature coverage

> NOTE: The terms 'ULP core' and 'LP core' are used interchangeably throughout this project.

- [x] ULP firmware loading
  - [x] Run once and halt
  - [x] Run on a loop
  - [x] Run from ULP Timer wake-up
  - [ ] Run from GPIO wake-up
- [x] Shared memory IPC between HP and LP 
  - [x] Locking (Mutex) of shared variables 
- [x] ULP Timer peripheral
  - [x] Start or stop the timer from either core
  - [x] Change the timer period from either core
- [x] ULP interrupt/exception handling
  - [x] Catch IllegalInstruction exceptions on the LP core
  - [x] Catch GPIO interrupts on the LP core
  - [ ] Catch LP interrupts on the HP core
- [x] HP sleep wakeup
  - [x] Light-sleep wakeup by LP core
  - [ ] Deep-sleep wakeup by LP core

# ESP-IDF test coverage

| Test # | Description                                        | Notes                                | ESP-IDF test? | Rust test? |
| -----: | -------------------------------------------------- | ------------------------------------ | :-----------: | :--------: |
|      1 | ULP and HP core simple data exchange               |                                      |     PASS      |    PASS    |
|      2 | ULP can wake HP from light sleep                   | ESP-IDF serial output missing?       |     PASS      |    PASS    |
|      3 | ULP can be stopped/resumed                         |                                      |     PASS      |    PASS    |
|      4 | ULP can run multiple firmwares                     | Somewhat redundant                   |     PASS      |    PASS    |
|      5 | ULP firmware recovery after crash                  |                                      |     PASS      |    PASS    |
|      6 | ULP can stop itself, and be resumed by HP          |                                      |     PASS      |    PASS    |
|      7 | ULP-HP mutex                                       |                                      |     PASS      |    PASS    |
|      8 | ULP can wake HP from deep sleep, after long delay  | Requires re-attaching serial console |     PASS      |            |
|      9 | Test No. 8, but with RTC periph. powered on        | Does not work!!! :O                  |     FAIL      |            |
|     10 | ULP can wake HP from deep sleep, after short delay |                                      |     PASS      |            |
|     11 | Test No. 11, but with RTC periph. powered on       |                                      |     FAIL      |            |
|     12 | ULP interrupt can be handled via ISR on HP core    | Checks wakeup signal and trap signal |     PASS      |            |
|     13 | ULP ADC can init-deinit-init                       |                                      |     PASS      |            |
|     14 | ULP RISC-V RTC I2C read and write test             |                                      |    IGNORED    |            |

# Basic Usage

0. Install NixOS/Nix, so that you have `nix-shell` available.
1. Clone this repo
2. Open a terminal in the `hp-app` directory
3. Run `nix-shell` to enter the development environment, which uses [esp-rs-nix](https://github.com/leighleighleigh/esp-rs-nix).
4. Plug in your `esp32s3` in development mode.
5. Inspect the `justfile`, make sure you are happy with it. Visit [just.systems](https://just.systems/) to learn more.
6. Run `just test`, which will...
 - Build the `lp-app` crate, copying the resulting binary into the `hp-app` crate.
 - Build the `hp-app` crate
 - Run the `integration.rs` tests using `probe-rs`

The output should be similar to the following...

```shell
(esp-hal-ulp-tests)leigh@leigh-desktop:hp-app$ just test
    Finished `release` profile [optimized] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/x86_64-unknown-linux-gnu/debug/cli -f /home/leigh/Git/esp-hal-ulp-test-crates/lp-app/target/riscv32imc-unknown-none-elf/release/app -l -b 0x50000000`
   Compiling hp-app v0.2.0 (/home/leigh/Git/esp-hal-ulp-test-crates/hp-app)
    Finished `test` profile [optimized + debuginfo] target(s) in 0.93s
     Running tests/integration.rs (target/xtensa-esp32s3-none-elf/debug/deps/integration-de237c7b4bd42a09)
      Erasing ✔ 100% [####################] 256.00 KiB @ 190.60 KiB/s (took 1s)
     Finished in 3.12s

running 15 tests
test tests::creating_peripheral_does_not_break_debug_connection ... ok
test tests::ipc_mutex_lock_test                                 ... ok
test tests::ulp_loop_oneshot                                    ... ok
test tests::ulp_interrupt_test                                  ... ok
test tests::ipc_xor_test                                        ... ok
test tests::ulp_loop_counter                                    ... ok
test tests::ulp_timer_counter                                   ... ok
test tests::hp_can_pause_ulp_timer                              ... ok
test tests::ulp_can_boot                                        ... ok
test tests::ulp_gpio_interrupt_test                             ... ok
test tests::hp_can_change_ulp_timer_period                      ... ok
test tests::hp_light_sleep_wakeup_by_ulp                        ... ok
test tests::ulp_can_stop_itself_then_resumed_by_hp              ... ok
test tests::ulp_exception_test                                  ... ok
test tests::ulp_can_change_timer_period                         ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.38s
```
