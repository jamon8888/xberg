#!/usr/bin/env bash

# Common utility functions for publish scripts
#
# This module provides shared functionality for npm publishing scripts:
# - Error handling and cleanup
# - Robust publish logic with idempotent version detection
# - Structured logging
#
# Usage: source "${SCRIPT_DIR}/lib/common.sh"

set -euo pipefail

# Setup: Export script directory for sourced scripts
SCRIPT_DIR="${SCRIPT_DIR:-.}"

# Color codes for terminal output
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_RESET='\033[0m'

# Global cleanup stack for EXIT trap
declare -a CLEANUP_STACK=()

# Register a cleanup handler to run on EXIT
# Usage: register_cleanup "rm -f $tmpfile"
register_cleanup() {
	local handler="$1"
	CLEANUP_STACK+=("$handler")
}

# Execute all registered cleanups in reverse order (LIFO)
_run_cleanups() {
	local i
	for ((i = ${#CLEANUP_STACK[@]} - 1; i >= 0; i--)); do
		eval "${CLEANUP_STACK[i]}" || true
	done
}

# Install EXIT trap for cleanup
trap '_run_cleanups' EXIT

# Log a message with timestamp
# Usage: log_info "Building package..."
log_info() {
	local msg="$1"
	printf "[INFO] %s: %s\n" "$(date '+%H:%M:%S')" "$msg"
}

# Log an error message
# Usage: log_error "Failed to publish package"
log_error() {
	local msg="$1"
	printf "${COLOR_RED}[ERROR] %s: %s${COLOR_RESET}\n" "$(date '+%H:%M:%S')" "$msg" >&2
}

# Log a warning message
# Usage: log_warning "Package already published"
log_warning() {
	local msg="$1"
	printf "${COLOR_YELLOW}[WARN] %s: %s${COLOR_RESET}\n" "$(date '+%H:%M:%S')" "$msg"
}

# Log success message
# Usage: log_success "Package published"
log_success() {
	local msg="$1"
	printf "${COLOR_GREEN}[OK] %s: %s${COLOR_RESET}\n" "$(date '+%H:%M:%S')" "$msg"
}

# Validate that a directory exists
# Usage: validate_directory "/path/to/dir" "npm directory"
# Returns: 0 if valid, exits with code 1 if not
validate_directory() {
	local path="$1"
	local name="$2"

	if [ ! -d "$path" ]; then
		log_error "$name not found: $path"
		exit 1
	fi
}

# Validate that a file exists
# Usage: validate_file "/path/to/file" "package.json"
# Returns: 0 if valid, exits with code 1 if not
validate_file() {
	local path="$1"
	local name="$2"

	if [ ! -f "$path" ]; then
		log_error "$name not found: $path"
		exit 1
	fi
}

# Check if a package version was already published (npm-specific)
# Examines the output log from npm publish for known version-already-published patterns
# Usage: is_already_published_npm "$publish_log"
# Returns: 0 if already published, 1 otherwise
is_already_published_npm() {
	local log_file="$1"

	# Match patterns that indicate the version is already published:
	# - "previously published versions"
	# - "You cannot publish over the previously published version"
	# - "cannot publish to repository"
	# These are brittle grep patterns; we improve with case-insensitive matching
	if grep -qi "previously published" "$log_file" ||
		grep -qi "cannot publish over" "$log_file" ||
		grep -qi "cannot publish to repository" "$log_file"; then
		return 0
	fi
	return 1
}

# Publish an npm package with idempotent version handling
# Handles already-published versions gracefully without failing
# CRITICAL: Supports npm dist-tag to prevent pre-release versions from being tagged 'latest'
#
# Usage: publish_npm_package "/path/to/package.tgz" [npm_tag]
# Arguments:
#   $1: Path to package.tgz file
#   $2: npm dist-tag (optional, defaults to 'latest')
# Returns: 0 on success or already-published, 1 on error
publish_npm_package() {
	local pkg_path="$1"
	local npm_tag="${2:-latest}"
	local pkg_name

	if [ ! -f "$pkg_path" ]; then
		log_error "Package file not found: $pkg_path"
		return 1
	fi

	pkg_name="$(basename "$pkg_path")"
	log_info "Publishing $pkg_name with tag '$npm_tag'"

	# Create temporary log file and register cleanup
	local publish_log
	publish_log=$(mktemp) || {
		log_error "Failed to create temporary log file"
		return 1
	}
	register_cleanup "rm -f '$publish_log'"

	# Execute publish, capture output and exit status
	# CRITICAL: Use --tag flag to control dist-tag (prevents pre-releases from being 'latest')
	local status
	set +e
	npm publish "$pkg_path" --access public --provenance --ignore-scripts --tag "$npm_tag" 2>&1 | tee "$publish_log"
	status=${PIPESTATUS[0]}
	set -e

	# Handle publish result
	if [ "$status" -eq 0 ]; then
		log_success "$pkg_name published to npm with tag '$npm_tag'"
		return 0
	fi

	# Check for already-published error
	if is_already_published_npm "$publish_log"; then
		log_warning "$pkg_name already published; skipping"
		return 0
	fi

	# Publish failed and it's not a version-already-published error
	log_error "Failed to publish $pkg_name"
	return 1
}

# Publish a package from a directory (using cd)
# Calls npm publish from within the specified directory
# CRITICAL: Supports npm dist-tag to prevent pre-release versions from being tagged 'latest'
#
# Usage: publish_npm_from_directory "/path/to/pkg" [npm_tag]
# Arguments:
#   $1: Path to package directory
#   $2: npm dist-tag (optional, defaults to 'latest')
# Returns: 0 on success or already-published, 1 on error
publish_npm_from_directory() {
	local pkg_dir="$1"
	local npm_tag="${2:-latest}"
	local pkg_name

	validate_directory "$pkg_dir" "Package directory"

	pkg_name="$(basename "$pkg_dir")"
	log_info "Publishing from $pkg_name with tag '$npm_tag'"

	# Create temporary log file and register cleanup
	local publish_log
	publish_log=$(mktemp) || {
		log_error "Failed to create temporary log file"
		return 1
	}
	register_cleanup "rm -f '$publish_log'"

	# Change to package directory and execute publish
	# CRITICAL: Use --tag flag to control dist-tag (prevents pre-releases from being 'latest')
	local status
	set +e
	(
		cd "$pkg_dir" || exit 1
		npm publish --access public --provenance --ignore-scripts --tag "$npm_tag" 2>&1 | tee "$publish_log"
	)
	status=${PIPESTATUS[0]}
	set -e

	# Handle publish result
	if [ "$status" -eq 0 ]; then
		log_success "$pkg_name published to npm with tag '$npm_tag'"
		return 0
	fi

	# Check for already-published error
	if is_already_published_npm "$publish_log"; then
		log_warning "$pkg_name already published; skipping"
		return 0
	fi

	# Publish failed and it's not a version-already-published error
	log_error "Failed to publish $pkg_name"
	return 1
}
