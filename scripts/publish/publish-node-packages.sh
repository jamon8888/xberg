#!/usr/bin/env bash

# Publish native binary packages to npm
#
# Publishes all packed npm packages (*.tgz files) from the npm directory.
# Includes idempotent handling for already-published versions.
# CRITICAL: Respects npm dist-tag to prevent pre-release versions from being tagged 'latest'
#
# Arguments:
#   $1: Directory containing npm packages (default: crates/kreuzberg-node/npm)
#   $2: npm dist-tag (default: latest)
#
# Environment Variables:
#   - NPM_TAG: Override npm dist-tag (optional)

set -euo pipefail

# Locate this script and its library directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

npm_dir="${1:-crates/kreuzberg-node/npm}"
npm_tag="${2:-${NPM_TAG:-latest}}"

# Validate npm directory exists
validate_directory "$npm_dir" "npm directory"

# Find all .tgz packages
shopt -s nullglob
pkgs=("$npm_dir"/*.tgz)

if [ ${#pkgs[@]} -eq 0 ]; then
	log_error "No npm packages found in $npm_dir"
	exit 1
fi

log_info "Found ${#pkgs[@]} package(s) to publish with tag '$npm_tag'"

# Publish each package with idempotent error handling
for pkg in "${pkgs[@]}"; do
	if ! publish_npm_package "$pkg" "$npm_tag"; then
		exit 1
	fi
done

log_success "Native binary packages published"
