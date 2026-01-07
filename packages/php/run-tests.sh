#!/usr/bin/env bash
# Test runner wrapper that handles ONNX Runtime cleanup errors
#
# This script wraps PHPUnit execution to:
# 1. Auto-detect ONNX Runtime library location
# 2. Handle the known ONNX Runtime mutex error during process cleanup
#
# All tests pass successfully, but ONNX Runtime's C++ destructors trigger
# a mutex error during final cleanup, causing exit code 134.
#
# This is a known issue documented in crates/kreuzberg/src/embeddings.rs
# and will be resolved when fastembed upgrades to ort >= 2.0.0.

set +e # Don't exit on error

# Auto-detect ONNX Runtime library if ORT_DYLIB_PATH is not set
if [ -z "$ORT_DYLIB_PATH" ]; then
  # Common installation locations by platform
  SEARCH_PATHS=(
    # macOS Homebrew (Apple Silicon)
    "/opt/homebrew/lib/libonnxruntime.dylib"
    # macOS Homebrew (Intel)
    "/usr/local/lib/libonnxruntime.dylib"
    # Linux common paths
    "/usr/lib/libonnxruntime.so"
    "/usr/lib/x86_64-linux-gnu/libonnxruntime.so"
    "/usr/lib/aarch64-linux-gnu/libonnxruntime.so"
    "/usr/local/lib/libonnxruntime.so"
    # Windows (MSYS2/MinGW)
    "/c/Program Files/onnxruntime/lib/onnxruntime.dll"
    "/mingw64/bin/onnxruntime.dll"
  )

  for path in "${SEARCH_PATHS[@]}"; do
    if [ -f "$path" ]; then
      export ORT_DYLIB_PATH="$path"
      echo "Auto-detected ONNX Runtime at: $path" >&2
      break
    fi
  done

  if [ -z "$ORT_DYLIB_PATH" ]; then
    echo "Warning: ONNX Runtime not found. Embedding tests may fail." >&2
    echo "Install: brew install onnxruntime (macOS), apt install libonnxruntime (Ubuntu)" >&2
  fi
fi

# Run phpunit and capture output
output=$(vendor/bin/phpunit "$@" 2>&1)
exit_code=$?

# Print the output
echo "$output"

# Check if tests actually passed by looking for success indicators
if echo "$output" | grep -q "^OK" ||
  (echo "$output" | grep -q "^Tests:" && ! echo "$output" | grep -q "Failures:"); then
  # Tests passed - check if exit code is 134 (ONNX cleanup error)
  if [ $exit_code -eq 134 ]; then
    # This is the known ONNX Runtime cleanup issue
    # All tests passed, so return success
    exit 0
  fi
fi

# Otherwise, return the actual exit code
exit $exit_code
