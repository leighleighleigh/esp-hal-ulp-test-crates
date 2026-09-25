# Override these variables in the crate-level justfile.
# bin_arch := "riscv32-esp-elf"

# Default arguments to make the outputs nicer for certain tools
objdump_defaults := "--demangle=rust --visualize-jumps=color --disassembler-color=on"
readelf_defaults := "--demangle=rust -W"
size_defaults := ""
nm_defaults := "--demangle=rust"

# Get the binary name for the current crate
crate-binary-name:
  #!/usr/bin/env bash
  set +eou pipefail
  cargo metadata --no-deps --format-version=1 | jq '.packages[0].targets | .[] | select(.kind[0] == "bin") | .name' -r

# Finds the last-built application file
find-last-built-app-elf NAME="":
  #!/usr/bin/env bash
  set +eou pipefail
  # If a name was not provided, run
  # 'just crate-binary-name' to figure it out.
  appname="{{NAME}}"
  if ! [ -n "${appname}" ]; then
    appname="$(just crate-binary-name)"
  fi
  elf_path="$(find ./target/ -maxdepth 5 -name "${appname}" -type f -printf "%A@ %p\n" | sort -r | head -n1 | cut -d' ' -f2)"
  realpath "${elf_path}"

# General-purpose GNU Binutils recipe,
# to run programs on the latest ELF
[private]
binutil TOOL_NAME FILE *ARGS="":
  #!/usr/bin/env bash
  {{bin_arch}}-{{TOOL_NAME}} {{ARGS}} {{FILE}}

objdump FILE *ARGS="-d":
  #!/usr/bin/env bash
  just binutil objdump {{FILE}} {{objdump_defaults}} {{ARGS}}

objcopy FILE *ARGS="":
  #!/usr/bin/env bash
  just binutil objcopy {{FILE}} {{ARGS}}

readelf FILE *ARGS="":
  #!/usr/bin/env bash
  just binutil readelf {{FILE}} {{readelf_defaults}} {{ARGS}}

size FILE *ARGS="-A":
  #!/usr/bin/env bash
  just binutil size {{FILE}} {{size_defaults}} {{ARGS}}

nm FILE *ARGS="-An":
  #!/usr/bin/env bash
  just binutil nm {{FILE}} {{nm_defaults}} {{ARGS}}

