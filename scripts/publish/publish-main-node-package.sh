#!/usr/bin/env bash

# Publish main Node package to npm
#
# Publishes the main @kreuzberg/node package using npm from the package directory.
# Includes idempotent handling for already-published versions.
# CRITICAL: Respects npm dist-tag to prevent pre-release versions from being tagged 'latest'
#
# Arguments:
#   $1: Package directory (default: crates/kreuzberg-node)
#   $2: npm dist-tag (default: latest)
#
# Environment Variables:
#   - NPM_TAG: Override npm dist-tag (optional)

set -euo pipefail

# Locate this script and its library directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

pkg_dir="${1:-crates/kreuzberg-node}"
npm_tag="${2:-${NPM_TAG:-latest}}"

# Validate package directory exists
validate_directory "$pkg_dir" "Package directory"

# Publish from the package directory with idempotent error handling
# CRITICAL: Pass npm_tag to control dist-tag (prevents pre-releases from being 'latest')
if ! publish_npm_from_directory "$pkg_dir" "$npm_tag"; then
	exit 1
fi

log_success "@kreuzberg/node published to npm with tag '$npm_tag'"
