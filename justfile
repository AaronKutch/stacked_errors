# The empty default uses whatever cargo is already active, which can come from the nix shell or
# rustup default
#
# - Nix: `nix develop .#nightly -c just check` (or `.#msrv`, or nothing for default pinned)
# - rustup: `just toolchain=nightly check` (or `toolchain=1.86`, etc.)
toolchain := ""
cargo := if toolchain == "" { "cargo" } else { "cargo +" + toolchain }
rustc := if toolchain == "" { "rustc" } else { "rustc +" + toolchain }

alias c := check
alias t := test
alias r := run

quick:
  {{cargo}} fmt
  {{cargo}} clippy --all --all-targets --all-features -- -D clippy::all

fix *ARGS:
  {{cargo}} clippy --fix --all --all-targets --all-features {{ARGS}} -- -D clippy::all

fmt:
  {{cargo}} sort -w
  {{cargo}} fmt

check:
  {{cargo}} check
  {{cargo}} clippy --all --all-targets -- -D clippy::all

test *ARGS:
  {{cargo}} nextest run --all-features {{ARGS}}

test_all:
  {{cargo}} sort -cw
  {{cargo}} doc --no-deps --all-features
  {{cargo}} nextest run
  {{cargo}} nextest run --release
  {{cargo}} t --doc --all-features
  {{cargo}} machete

run *ARGS:
  {{cargo}} r --bin {{ARGS}}

doc *ARGS:
  {{cargo}} doc --open {{ARGS}}

clean:
  {{cargo}} clean

# Print the nix shell's PATH, for VSCode for instance you can add this to get rust-analyzer to work:
# `"rust-analyzer.cargo.extraEnv": {"NIX_PROFILES": "/nix/var/nix/profiles/default ${userHome}/.nix-profile", "PATH": "..."},`
ra_path:
  nix develop .#nightly --command printenv PATH

# equivalent to `rustup doc`
std_doc:
  xdg-open "$({{rustc}} --print sysroot)/share/doc/rust/html/index.html"
