###### LINTING ######

format:
  #!/usr/bin/env bash
  shopt -s globstar 
  cargo fmt --all -- --config-path=rustfmt.toml src/**/*.rs

clippy *CLIPPY_ARGS="":
  #!/usr/bin/env bash
  cargo clippy --release {{CLIPPY_ARGS}} -- -D warnings --no-deps