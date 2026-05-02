###### Common recipes for HP-core crates ######

## ! Override these parameters in the crate-level justfile !

# chip - The ESP32 chip to target, for both HP and LP crates.
chip := "esp32s3" 

# ulp_app_crate - The crate containing the ULP project.
# This will be built using the command `cargo {{chip}}`,
# which must be aliased in the `.cargo/config.toml` of the ULP project.
ulp_app_crate := "../ulp-baremetal/"  

# espflash_flags - Extra arguments to use when flashing the HP core.
espflash_flags := "--no-skip --ignore_app_descriptor --partition-table ./partitions.csv"

[doc('Build LP and HP crates, flash chip, and monitor serial')]
all:
  #!/usr/bin/env bash
  set -eou pipefail
  just build && just flash && just monitor 

[doc('Build LP & HP crates')]
build ULPAPP=ulp_app_crate CHIP=chip:
  #!/usr/bin/env bash
  set -eou pipefail
  just build-ulp-app {{ULPAPP}} {{CHIP}} && \
  cargo {{CHIP}}

# Enters the ulp project, and builds it.
[private]
build-ulp-app APPCRATE CHIP:
  #!/usr/bin/env bash
  set -eou pipefail
  # Move into the ULP/LP app crate directory
  appdir="$(realpath {{APPCRATE}})"
  pushd "${appdir}" >/dev/null
  # Run the build alias, and save the return code
  cargo {{CHIP}}
  ecode="$?"
  # Get the absolute path to the built ELF,
  # and return to the HP crate directory
  appfile="$(just find-last-built-app-elf)"
  popd >/dev/null
  # Copy the built application to lp_app
  outfile="$(realpath ./)/lp_app"
  cp -v "${appfile}" "${outfile}"
  exit $ecode

# Repeatedly attempts to flash until successful
flash APPFILE=`just find-last-built-app-elf`:
  #!/usr/bin/env bash
  set -x
  while [ true ]; do
    espflash flash {{espflash_flags}} -p /dev/ttyACM* {{APPFILE}} && exit 0
    sleep 0.5
  done

# Stream serial output, will re-attempt connection if lost.
monitor PORT="/dev/ttyACM*":
  #!/usr/bin/env bash
  while [ true ]; do
    test -c {{PORT}} && just try-monitor {{PORT}}
    # if exit with Ctrl-C, return code will be 130.
    # this should cause a break in the loop
    if [ "$?" -eq 130 ];
    then
      break
    fi
  done

[private]
try-monitor PORT:
  #!/usr/bin/env bash
  echo "# [$(date +%s)] Connected to {{PORT}}"
  while [ true ]; do
    stty -F {{PORT}} speed 115200 cs8 -cstopb -parenb -echo time 3 min 0 2>/dev/null && break
    sleep 0.01
  done
  while [ true ]; do
    cat {{PORT}} 2>/dev/null || break
  done
  echo "# [$(date +%s)] Disconnected from {{PORT}}"
