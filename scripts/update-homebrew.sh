#!/usr/bin/env bash
set -euo pipefail

# Updates the Homebrew formula in geekdada/homebrew-tap for a given release version.
# Usage: ./scripts/update-homebrew.sh <version>
# Example: ./scripts/update-homebrew.sh 0.2.0
#
# Expects release assets to already exist at:
#   https://github.com/geekdada/models-dev-cli/releases/download/v<version>/models-<target>.tar.gz

REPO="geekdada/models-dev-cli"
TAP_REPO="geekdada/homebrew-tap"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <version>"
  exit 1
fi

VERSION="$1"
TAG="v${VERSION}"
BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"

TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "aarch64-unknown-linux-gnu"
  "x86_64-unknown-linux-gnu"
)

declare -A SHAS

for target in "${TARGETS[@]}"; do
  asset="models-${target}.tar.gz"
  url="${BASE_URL}/${asset}"
  echo "Downloading ${asset}..."
  sha=$(curl -sL "$url" | shasum -a 256 | awk '{print $1}')
  SHAS["$target"]="$sha"
  echo "  SHA256: $sha"
done

FORMULA=$(cat <<RUBY
class Models < Formula
  desc "Terminal UI for browsing AI model data from models.dev"
  homepage "https://github.com/${REPO}"
  version "${VERSION}"
  license "MIT"

  on_macos do
    on_arm do
      url "${BASE_URL}/models-aarch64-apple-darwin.tar.gz"
      sha256 "${SHAS[aarch64-apple-darwin]}"
    end
    on_intel do
      url "${BASE_URL}/models-x86_64-apple-darwin.tar.gz"
      sha256 "${SHAS[x86_64-apple-darwin]}"
    end
  end

  on_linux do
    on_arm do
      url "${BASE_URL}/models-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${SHAS[aarch64-unknown-linux-gnu]}"
    end
    on_intel do
      url "${BASE_URL}/models-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${SHAS[x86_64-unknown-linux-gnu]}"
    end
  end

  def install
    bin.install "models"
  end
end
RUBY
)

echo ""
echo "Generated formula:"
echo "$FORMULA"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo ""
echo "Cloning ${TAP_REPO}..."
gh repo clone "$TAP_REPO" "$TMPDIR/homebrew-tap" -- --depth 1

mkdir -p "$TMPDIR/homebrew-tap/Formula"
echo "$FORMULA" > "$TMPDIR/homebrew-tap/Formula/models.rb"

cd "$TMPDIR/homebrew-tap"
git add Formula/models.rb
if git diff --cached --quiet; then
  echo "No changes to formula."
else
  git commit -m "Update models to ${VERSION}"
  git push
  echo "Formula pushed to ${TAP_REPO}."
fi
