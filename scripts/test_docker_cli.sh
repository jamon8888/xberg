#!/usr/bin/env bash
# Docker CLI image tests
# Tests the minimal kreuzberg-cli Docker image (Alpine + static musl binary)
#
# Usage:
#   ./scripts/test_docker_cli.sh [--skip-build] [--image IMAGE] [--verbose]

set -euo pipefail

IMAGE_NAME="${IMAGE_NAME:-kreuzberg:cli}"
SKIP_BUILD=false
VERBOSE=false
CONTAINER_PREFIX="kreuzberg-cli-test"
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_DOCS_DIR="${TEST_DIR}/test_documents"
TEST_RESULTS_FILE="/tmp/kreuzberg-docker-cli-test-results.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
declare -a FAILED_TEST_NAMES=()

while [[ $# -gt 0 ]]; do
  case $1 in
  --skip-build)
    SKIP_BUILD=true
    shift
    ;;
  --image)
    IMAGE_NAME="$2"
    shift 2
    ;;
  --verbose)
    VERBOSE=true
    shift
    ;;
  *)
    echo "Unknown option: $1"
    echo "Usage: $0 [--skip-build] [--image IMAGE] [--verbose]"
    exit 1
    ;;
  esac
done

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_verbose() {
  if [ "$VERBOSE" = true ]; then
    echo -e "${YELLOW}[VERBOSE]${NC} $*"
  fi
}

start_test() {
  TOTAL_TESTS=$((TOTAL_TESTS + 1))
  log_info "Test $TOTAL_TESTS: $*"
}

pass_test() {
  PASSED_TESTS=$((PASSED_TESTS + 1))
  log_success "PASS"
}

fail_test() {
  FAILED_TESTS=$((FAILED_TESTS + 1))
  FAILED_TEST_NAMES+=("$1")
  log_error "FAIL: $1"
  if [ -n "${2:-}" ]; then
    log_error "  Details: $2"
  fi
}

# shellcheck disable=SC2317,SC2329
cleanup() {
  log_info "Cleaning up test containers..."
  docker ps -a --filter "name=${CONTAINER_PREFIX}" --format "{{.Names}}" | while read -r container; do
    docker rm -f "$container" 2>/dev/null || true
  done
}

trap cleanup EXIT

random_container_name() {
  echo "${CONTAINER_PREFIX}-$(date +%s)-${RANDOM}"
}

# --- Build ---

if [ "$SKIP_BUILD" = false ]; then
  log_info "Building Docker CLI image: $IMAGE_NAME"
  docker build -f docker/Dockerfile.cli -t "$IMAGE_NAME" "$TEST_DIR" || {
    log_error "Docker build failed"
    exit 1
  }
  log_success "Docker build completed"
else
  log_warning "Skipping Docker build (--skip-build flag set)"
fi

if ! docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${IMAGE_NAME}$"; then
  log_error "Docker image $IMAGE_NAME not found"
  exit 1
fi

log_info "Starting Docker CLI tests for: $IMAGE_NAME"
echo "========================================================================"

# --- Tests ---

start_test "Docker image exists"
if docker inspect "$IMAGE_NAME" >/dev/null 2>&1; then
  pass_test
else
  fail_test "Image does not exist" "$IMAGE_NAME"
fi

start_test "Image size is reasonable (< 200MB)"
size_mb=$(docker inspect "$IMAGE_NAME" --format='{{.Size}}' | awk '{print int($1/1024/1024)}')
log_verbose "Image size: ${size_mb}MB"
if [ "$size_mb" -lt 200 ]; then
  pass_test
else
  fail_test "Image size" "Expected < 200MB, got ${size_mb}MB"
fi

start_test "CLI --version command"
output=$(docker run --rm "$IMAGE_NAME" --version 2>&1 || true)
log_verbose "Version output: $output"
if echo "$output" | grep -qi "kreuzberg"; then
  pass_test
else
  fail_test "CLI version" "Expected 'kreuzberg' in output, got: $output"
fi

start_test "CLI --help command"
output=$(docker run --rm "$IMAGE_NAME" --help 2>&1 || true)
if echo "$output" | grep -qi "extract"; then
  pass_test
else
  fail_test "CLI help" "Expected 'extract' in help output"
fi

start_test "MIME type detection (detect command)"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  detect /data/pdf/searchable.pdf 2>&1 || true)
log_verbose "MIME detection output: $output"
if echo "$output" | grep -qi "application/pdf"; then
  pass_test
else
  fail_test "MIME detection" "Expected 'application/pdf', got: $output"
fi

start_test "Extract plain text file"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  extract /data/text/contract.txt 2>&1 || true)
log_verbose "Text extraction output (first 100 chars): ${output:0:100}"
text_length=${#output}
if [ "$text_length" -gt 15 ] && echo "$output" | grep -qi "contract"; then
  pass_test
else
  fail_test "Text extraction" "Output too short (${text_length} chars) or missing expected keywords"
fi

start_test "Extract searchable PDF"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  extract /data/pdf/searchable.pdf 2>&1 || true)
log_verbose "PDF extraction output (first 100 chars): ${output:0:100}"
if [ ${#output} -gt 50 ]; then
  pass_test
else
  fail_test "Searchable PDF extraction" "Output too short: ${#output} chars"
fi

start_test "Extract HTML file"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  extract /data/html/simple_table.html 2>&1 || true)
log_verbose "HTML extraction output (first 100 chars): ${output:0:100}"
if [ ${#output} -gt 10 ]; then
  pass_test
else
  fail_test "HTML extraction" "Output too short: ${#output} chars"
fi

start_test "Extract DOCX file"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  extract /data/docx/extraction_test.docx 2>&1 || true)
log_verbose "DOCX extraction output (first 100 chars): ${output:0:100}"
docx_length=${#output}
if [ "$docx_length" -gt 100 ]; then
  pass_test
else
  fail_test "DOCX extraction" "Output too short (${docx_length} chars)"
fi

start_test "Batch extraction (multiple files)"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  batch /data/text/contract.txt /data/html/simple_table.html 2>&1 || true)
log_verbose "Batch output (first 200 chars): ${output:0:200}"
if [ ${#output} -gt 20 ]; then
  pass_test
else
  fail_test "Batch extraction" "Output too short: ${#output} chars"
fi

start_test "JSON output format"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  "$IMAGE_NAME" \
  extract /data/text/simple.txt --output-format json 2>&1 || true)
log_verbose "JSON output (first 200 chars): ${output:0:200}"
if echo "$output" | grep -q '{'; then
  pass_test
else
  fail_test "JSON output" "Expected JSON object in output"
fi

start_test "Read-only volume mount works"
container=$(random_container_name)
output=$(docker run --rm \
  --name "$container" \
  -v "${TEST_DOCS_DIR}:/data:ro" \
  --read-only \
  --tmpfs /tmp \
  "$IMAGE_NAME" \
  extract /data/text/simple.txt 2>&1 || true)
if [ ${#output} -gt 5 ]; then
  pass_test
else
  fail_test "Read-only mount" "Failed to extract with read-only filesystem"
fi

start_test "Non-existent file returns error"
container=$(random_container_name)
exit_code=0
docker run --rm \
  --name "$container" \
  "$IMAGE_NAME" \
  extract /nonexistent/file.pdf >/dev/null 2>&1 || exit_code=$?
if [ "$exit_code" -ne 0 ]; then
  pass_test
else
  fail_test "Error on missing file" "Expected non-zero exit code for missing file"
fi

# --- Results ---

echo ""
echo "========================================================================"
log_info "Test Results: $PASSED_TESTS/$TOTAL_TESTS passed, $FAILED_TESTS failed"
echo "========================================================================"

if [ $FAILED_TESTS -gt 0 ]; then
  log_error "Failed tests:"
  for name in "${FAILED_TEST_NAMES[@]}"; do
    log_error "  - $name"
  done
fi

# Write results JSON
cat >"$TEST_RESULTS_FILE" <<EOF
{
  "variant": "cli",
  "image": "$IMAGE_NAME",
  "total": $TOTAL_TESTS,
  "passed": $PASSED_TESTS,
  "failed": $FAILED_TESTS,
  "failed_tests": [$(printf '"%s",' "${FAILED_TEST_NAMES[@]}" | sed 's/,$//')]
}
EOF

if [ $FAILED_TESTS -gt 0 ]; then
  exit 1
fi

log_success "All CLI Docker tests passed!"
