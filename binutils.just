# Override these variables in the crate-level justfile.
# bin_name := "app"
# bin_arch := "riscv32-esp-elf"

# Default arguments to make the outputs nicer for certain tools
objdump_defaults := "--demangle=rust --visualize-jumps=extended-color --disassembler-color=extended"
readelf_defaults := "--demangle=rust -W"
size_defaults := ""
nm_defaults := "--demangle=rust"

# Finds the last-built application file
find-last-built-app-elf:
  #!/usr/bin/env bash
  set +eou pipefail
  elf_path="$(find ./target/ -maxdepth 3 -name "{{bin_name}}" -type f -printf "%A@ %p\n" | sort -r | head -n1 | cut -d' ' -f2)"
  realpath "${elf_path}"

# General-purpose GNU Binutils recipe,
# to run programs on the latest ELF
[private]
binutil TOOL_NAME *ARGS="":
  #!/usr/bin/env bash
  elffile="$(just find-last-built-app-elf)"
  {{bin_arch}}-{{TOOL_NAME}} "${elffile}" {{ARGS}}

objdump *ARGS="-d":
  #!/usr/bin/env bash
  just binutil objdump {{objdump_defaults}} {{ARGS}}

objcopy *ARGS="":
  #!/usr/bin/env bash
  just binutil objcopy {{ARGS}}

readelf *ARGS="":
  #!/usr/bin/env bash
  just binutil readelf {{readelf_defaults}} {{ARGS}}

size *ARGS="-A":
  #!/usr/bin/env bash
  just binutil size {{size_defaults}} {{ARGS}}

nm *ARGS="-An":
  #!/usr/bin/env bash
  just binutil nm {{nm_defaults}} {{ARGS}}

