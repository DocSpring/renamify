#!/usr/bin/env bash
# dev-setup.sh - Install required Rust dev tools for this repo.
# - Installs via cargo-binstall if available (faster prebuilt), else falls back to cargo install.
# - Idempotent: skips tools already at the requested version.
# Customize tools by editing TOOLS below or exporting TOOL_LIST='name@ver name2@ver'

set -euo pipefail

# -------- Config --------
: "${TOOL_LIST:=cargo-insta@1.43.0 cargo-nextest@0.9.91}"

# -------- Helpers --------
log() { printf "\033[1;34m[dev-setup]\033[0m %s\n" "$*"; }
err() { printf "\033[1;31m[dev-setup]\033[0m %s\n" "$*" >&2; }
have() { command -v "$1" >/dev/null 2>&1; }

need_rust() {
  if ! have cargo; then
    log "Rust toolchain not found. Installing rustup + stable toolchain..."
    # Official rustup installer (non-interactive)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi
}

ensure_binstall() {
  if have cargo-binstall; then return; fi
  log "cargo-binstall not found - optional but faster. Attempting install..."
  # Try to install prebuilt cargo-binstall
  if curl -fsSL https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
    | bash -s -- -y; then
    log "Installed cargo-binstall."
  else
    err "cargo-binstall install failed - will fall back to 'cargo install'."
  fi
}

installed_version() {
  local bin="$1"
  if ! have "$bin"; then return 1; fi
  # Expect outputs like "cargo-insta 1.43.0"
  "$bin" --version 2>/dev/null | awk '{print $NF}'
}

install_tool() {
  local spec="$1"        # name@version
  local name="${spec%@*}"
  local ver="${spec#*@}"
  local bin="$name"      # cargo subcommands install as this name

  # nextest binary is "cargo-nextest" but the crate is "nextest"
  local crate="$name"
  if [[ "$name" == "cargo-nextest" ]]; then crate="cargo-nextest"; fi

  if curver="$(installed_version "$bin")"; then
    if [[ "$curver" == "$ver" ]]; then
      log "$name $ver already installed. Skipping."
      return
    else
      log "$name present at $curver - upgrading to $ver"
    fi
  else
    log "Installing $name $ver..."
  fi

  if have cargo-binstall; then
    cargo binstall -y "$crate@$ver" || {
      err "cargo-binstall failed for $crate@$ver - falling back to cargo install."
      cargo install --locked --version "$ver" "$crate"
    }
  else
    cargo install --locked --version "$ver" "$crate"
  fi
}

# -------- Run --------
need_rust
ensure_binstall

for spec in $TOOL_LIST; do
  install_tool "$spec"
done

log "Verifying installs..."
for spec in $TOOL_LIST; do
  bin="${spec%@*}"
  "$bin" --version
done

log "Done. If ~/.cargo/bin is not on your PATH, add it now:"
echo '  export PATH="$HOME/.cargo/bin:$PATH"'
