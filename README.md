# esp-hal-ulp-test-crates

I've been slowly porting the `esp-idf` [ULP RISC-V test suite](https://github.com/espressif/esp-idf/blob/master/components/ulp/test_apps/ulp_riscv/main/test_ulp_riscv.c) to Rust, while I work on adding new features to `esp-lp-hal`.  

# Feature coverage

(TODO)

# ESP-IDF test coverage

| `esp-idf` test | Description                                        | Notes                                                                         | ESP-IDF test? | Rust test? |
| -------------: | -------------------------------------------------- | ----------------------------------------------------------------------------- | :-----------: | :--------: |
|              1 | ULP and HP core simple data exchange               |                                                                               |     PASS      |    PASS    |
|              2 | ULP can wake HP from light sleep                   | Serial output for the test result is missing?                                 |     PASS      |    PASS    |
|              3 | ULP can be stopped/resumed                         |                                                                               |     PASS      |    PASS    |
|              4 | ULP can run multiple firmwares                     | HP stops the ULP timer & halts ULP core, re-programs it with another firmware |     PASS      |    PASS    |
|              5 | ULP firmware recovery after crash                  |                                                                               |     PASS      |    PASS    |
|              6 | ULP can stop itself, and be resumed by HP          |                                                                               |     PASS      |    PASS    |
|              7 | ULP-HP mutex                                       |                                                                               |     PASS      |    PASS    |
|              8 | ULP can wake HP from deep sleep, after long delay  | Requires re-attaching serial console                                          |     PASS      |            |
|              9 | Test No. 8, but with RTC periph. powered on        | Does not work!!! :O                                                           |     FAIL      |            |
|             10 | ULP can wake HP from deep sleep, after short delay |                                                                               |     PASS      |            |
|             11 | Test No. 11, but with RTC periph. powered on       |                                                                               |     FAIL      |            |
|             12 | ULP interrupt can be handled via ISR on HP core    | Checks wakeup signal and trap signal                                          |     PASS      |            |
|             13 | ULP ADC can init-deinit-init                       | Would be cool if this printed the measured value                              |     PASS      |            |
|             14 | ULP RISC-V RTC I2C read and write test             |                                                                               |    IGNORED    |            |

# Basic Usage

0. Install NixOS/Nix, so that you have `nix-shell` available.
1. Clone this repo
2. Open a terminal in the `testsuite/hp-test` directory
3. Run `nix-shell` to enter the development environment, which uses [esp-rs-nix](https://github.com/leighleighleigh/esp-rs-nix).
4. Plug in your `esp32s3` in development mode.
5. Inspect the `justfile`, make sure you are happy with it. Visit [just.systems](https://just.systems/) to learn more.
6. Run `just test`, which will...
 - Build the `ulp-app-main` crate, copying the resulting binary into the `hp-test` crate.
 - Build the `hp-test` crate
 - Run the `integration.rs` tests using `probe-rs`

The output should be similar to the following...

```shell
(esp-hal-ulp-tests)leigh@leigh-desktop:hp-test$ just test
    Finished `release` profile [optimized] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/x86_64-unknown-linux-gnu/debug/cli -f /home/leigh/Git/esp-hal-ulp-test-crates/testsuite/ulp-app-main/target/riscv32imc-unknown-none-elf/release/app -l -b 0x50000000`
   Compiling hp-test v0.2.0 (/home/leigh/Git/esp-hal-ulp-test-crates/testsuite/hp-test)
    Finished `test` profile [optimized + debuginfo] target(s) in 0.95s
     Running tests/integration.rs (target/xtensa-esp32s3-none-elf/debug/deps/integration-bd6c529570074ad2)
      Erasing ✔ 100% [####################] 256.00 KiB @ 191.82 KiB/s (took 1s)
  Programming ✔ 100% [####################]  74.51 KiB @  56.85 KiB/s (took 1s)                                                                                                                                         Finished in 3.13s

running 15 tests
test tests::creating_peripheral_does_not_break_debug_connection ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
ok
test tests::ipc_mutex_lock_test                                 ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
ok
test tests::ulp_loop_oneshot                                    ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
ok
test tests::ulp_interrupt_test                                  ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  1, b = 2, rate = 10 Hz (hil_test hp-test/src/ulp_utils.rs:176)
[DEBUG] ULP_DEBUG_TRAP_DATA = 0x00000000 (integration hp-test/tests/integration.rs:332)
[DEBUG] ULP_DEBUG_ISR_DATA = 0xcafebabe (integration hp-test/tests/integration.rs:334)
[DEBUG] interrupt wrote: 0xcafebabe (integration hp-test/tests/integration.rs:338)
[DEBUG] a =  2, b = 3, rate = 10 Hz (hil_test hp-test/src/ulp_utils.rs:176)
ok
test tests::ipc_xor_test                                        ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
ok
test tests::ulp_loop_counter                                    ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  169, b = 174, rate = 102040 Hz (hil_test hp-test/src/ulp_utils.rs:176)
ok
test tests::ulp_timer_counter                                   ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] count: 1 (integration hp-test/tests/integration.rs:161)
[DEBUG] count: 105 (integration hp-test/tests/integration.rs:166)
ok
test tests::hp_can_pause_ulp_timer                              ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  1, b = 2, rate = 115 Hz (hil_test hp-test/src/ulp_utils.rs:176)
[DEBUG] a =  2, b = 2. Timed out. (hil_test hp-test/src/ulp_utils.rs:172)
[DEBUG] a =  2, b = 2, rate = 0 Hz (hil_test hp-test/src/ulp_utils.rs:173)
[DEBUG] a =  2, b = 3, rate = 6493 Hz (hil_test hp-test/src/ulp_utils.rs:176)
ok
test tests::ulp_can_boot                                        ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
ok
test tests::ulp_gpio_interrupt_test                             ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  1, b = 2, rate = 10 Hz (hil_test hp-test/src/ulp_utils.rs:176)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: true, LP interrupted: true, LP input: true (integration hp-test/tests/integration.rs:420)
[DEBUG] HP output: false, LP interrupted: true, LP input: false (integration hp-test/tests/integration.rs:420)
[DEBUG] a =  2, b = 3, rate = 60 Hz (hil_test hp-test/src/ulp_utils.rs:176)
ok
test tests::hp_can_change_ulp_timer_period                      ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  1, b = 2, rate = 1 Hz (hil_test hp-test/src/ulp_utils.rs:176)
[DEBUG] count: 592 (integration hp-test/tests/integration.rs:212)
[DEBUG] count: 1 (integration hp-test/tests/integration.rs:220)
ok
test tests::hp_light_sleep_wakeup_by_ulp                        ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] UlpLock acquired. (integration hp-test/tests/integration.rs:518)
[DEBUG] ULP booted. (integration hp-test/tests/integration.rs:522)
[DEBUG] Entering light sleep... (integration hp-test/tests/integration.rs:532)
[DEBUG] slept for 277 µs (integration hp-test/tests/integration.rs:544)
ok
test tests::ulp_can_stop_itself_then_resumed_by_hp              ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  1, b = 1. Timed out. (hil_test hp-test/src/ulp_utils.rs:172)
[DEBUG] a =  1, b = 1, rate = 0 Hz (hil_test hp-test/src/ulp_utils.rs:173)
ok
test tests::ulp_exception_test                                  ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] ULP_DEBUG_TRAP_DATA0 = 0x00000000 (integration hp-test/tests/integration.rs:306)
[DEBUG] ULP_DEBUG_TRAP_DATA1: 0xdeadbeef (integration hp-test/tests/integration.rs:310)
[DEBUG] a =  1, b = 1. Timed out. (hil_test hp-test/src/ulp_utils.rs:172)
[DEBUG] a =  1, b = 1, rate = 0 Hz (hil_test hp-test/src/ulp_utils.rs:173)
ok
test tests::ulp_can_change_timer_period                         ... [DEBUG] 
 (integration hp-test/tests/integration.rs:95)
[DEBUG] a =  1, b = 2, rate = 881 Hz (hil_test hp-test/src/ulp_utils.rs:176)
[DEBUG] Waiting for 1 second... (integration hp-test/tests/integration.rs:244)
[DEBUG] count: 490 (integration hp-test/tests/integration.rs:247)
[DEBUG] Pausing timer... (integration hp-test/tests/integration.rs:251)
[DEBUG] a =  490, b = 490. Timed out. (hil_test hp-test/src/ulp_utils.rs:172)
[DEBUG] a =  490, b = 490, rate = 0 Hz (hil_test hp-test/src/ulp_utils.rs:173)
[DEBUG] Resuming timer... (integration hp-test/tests/integration.rs:262)
[DEBUG] Waiting for 1 second... (integration hp-test/tests/integration.rs:268)
[DEBUG] count: 3 (integration hp-test/tests/integration.rs:271)
ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.44s
```
