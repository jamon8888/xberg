#!/usr/bin/env bash

set -euo pipefail

artifacts_dir="${1:?Artifacts directory argument required}"
tap_dir="${2:-homebrew-tap}"
tag="${TAG:?TAG not set}"
version="${VERSION:?VERSION not set}"
dry_run="${DRY_RUN:-false}"

if [ ! -d "$artifacts_dir" ]; then
  echo "Error: Artifacts directory not found: $artifacts_dir" >&2
  exit 1
fi

echo "=== Updating Homebrew formula with bottles ==="
echo "Tag: $tag"
echo "Version: $version"
echo "Artifacts: $artifacts_dir"

declare -A bottle_hashes
declare -a bottle_tags

for bottle in "$artifacts_dir"/kreuzberg-*.bottle.tar.gz; do
  if [ -f "$bottle" ]; then
    filename="$(basename "$bottle")"
    without_suffix="${filename%.bottle.tar.gz}"
    bottle_tag="${without_suffix##*.}"
    sha256=$(shasum -a 256 "$bottle" | cut -d' ' -f1)

    bottle_hashes[$bottle_tag]=$sha256
    bottle_tags+=("$bottle_tag")
    echo "  $bottle_tag: $sha256"
  fi
done

if [ ${#bottle_hashes[@]} -eq 0 ]; then
  echo "Warning: No bottle artifacts found" >&2
  exit 1
fi

if [ ! -d "$tap_dir" ]; then
  echo "Cloning homebrew-tap..."
  git clone https://github.com/kreuzberg-dev/homebrew-tap.git "$tap_dir"
fi

formula_path="$tap_dir/Formula/kreuzberg.rb"

if [ ! -f "$formula_path" ]; then
  echo "Error: Formula not found at $formula_path" >&2
  exit 1
fi

formula_content=$(<"$formula_path")

# Fetch the SHA256 of the source tarball
echo "Fetching SHA256 of source tarball..."
tarball_url="https://github.com/kreuzberg-dev/kreuzberg/archive/$tag.tar.gz"
tarball_sha256=$(curl -sL "$tarball_url" | shasum -a 256 | cut -d' ' -f1)
echo "Source tarball SHA256: $tarball_sha256"

bottle_block="  bottle do"
bottle_block+=$'\n'"    root_url \"https://github.com/kreuzberg-dev/kreuzberg/releases/download/$tag\""

for bottle_tag in "${bottle_tags[@]}"; do
  sha256=${bottle_hashes[$bottle_tag]}
  bottle_block+=$'\n'"    sha256 cellar: :any_skip_relocation, $bottle_tag: \"$sha256\""
done

bottle_block+=$'\n'"  end"

# Update URL and sha256 (sha256 comes right after url line)
new_formula=$(echo "$formula_content" | sed \
  -e "s|url \"https://github.com/kreuzberg-dev/kreuzberg/archive/.*\.tar\.gz\"|url \"https://github.com/kreuzberg-dev/kreuzberg/archive/$tag.tar.gz\"|" \
  -e "s|sha256 \"[a-f0-9]*\"|sha256 \"$tarball_sha256\"|")

# Remove any existing bottle blocks (both commented and uncommented)
new_formula=$(echo "$new_formula" | sed '/^  bottle do$/,/^  end$/d')
new_formula=$(echo "$new_formula" | sed '/^  # bottle do$/,/^  # end$/d')

# Use Python for reliable multiline replacement since bash/sed/awk have issues with multiline variables
# Also removes extra blank lines and inserts the bottle block before first depends_on
new_formula=$(
  python3 <<PYTHON_SCRIPT
import re

formula = """$new_formula"""
bottle_block = """$bottle_block"""

# Remove multiple consecutive blank lines (keep max 1 blank line between sections)
lines = formula.split('\n')
result = []
prev_blank = False

for line in lines:
  is_blank = line.strip() == ''

  # Skip consecutive blank lines
  if is_blank and prev_blank:
    continue

  prev_blank = is_blank
  result.append(line)

# Now insert the bottle block before the first depends_on
final_result = []
inserted = False

for line in result:
  if line.startswith('  depends_on') and not inserted:
    # Insert bottle block before this line
    final_result.append(bottle_block)
    final_result.append('')
    inserted = True
  final_result.append(line)

print('\n'.join(final_result))
PYTHON_SCRIPT
)

echo "$new_formula" >"$formula_path"

echo ""
echo "=== Updated formula ==="
head -30 "$formula_path"
echo "..."

if [ "$dry_run" = "true" ]; then
  echo ""
  echo "Dry run mode: skipping git operations"
  echo "Formula would be updated at: $formula_path"
  exit 0
fi

cd "$tap_dir"
git config user.name "kreuzberg-bot"
git config user.email "bot@kreuzberg.dev"

if git diff --quiet Formula/kreuzberg.rb; then
  echo "No changes to formula"
  exit 0
fi

git add Formula/kreuzberg.rb
git commit -m "chore(homebrew): update kreuzberg to $version

Auto-update from release $tag

Includes pre-built bottles for macOS"

echo "Pushing to homebrew-tap..."
git push origin main

echo "Formula updated successfully"
