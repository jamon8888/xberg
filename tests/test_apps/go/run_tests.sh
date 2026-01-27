#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

export DYLD_LIBRARY_PATH="$REPO_ROOT/target/release:$DYLD_LIBRARY_PATH"
export LD_LIBRARY_PATH="$REPO_ROOT/target/release:$LD_LIBRARY_PATH"
export PKG_CONFIG_PATH="$REPO_ROOT/crates/kreuzberg-ffi:$PKG_CONFIG_PATH"

cd "$SCRIPT_DIR"
# Use kreuzberg_dev tag for development builds within monorepo
go test -tags kreuzberg_dev -v ./...
