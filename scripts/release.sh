#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh <version>"
  echo "Example: ./scripts/release.sh 0.2.1"
  exit 1
fi

TAG="v${VERSION}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Update version in tauri.conf.json
sed -i '' "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$ROOT_DIR/src-tauri/tauri.conf.json"

# Update version in Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" "$ROOT_DIR/src-tauri/Cargo.toml"

# Sync Cargo.lock with new version
(cd "$ROOT_DIR/src-tauri" && cargo generate-lockfile)

# Commit version bump if there are changes
if ! git diff --quiet; then
  git add "$ROOT_DIR/src-tauri/tauri.conf.json" "$ROOT_DIR/src-tauri/Cargo.toml" "$ROOT_DIR/src-tauri/Cargo.lock"
  git commit -m "chore: bump version to ${VERSION}"
fi

# Clean up existing release + tag completely
if gh release view "$TAG" &>/dev/null; then
  echo "Deleting existing release $TAG..."
  gh release delete "$TAG" --cleanup-tag --yes
  # Wait for GitHub to fully process the deletion
  sleep 3
fi

# Also clean up local/remote tag if it still exists
if git rev-parse "$TAG" &>/dev/null; then
  git tag -d "$TAG" 2>/dev/null || true
fi
git push origin ":refs/tags/$TAG" 2>/dev/null || true

echo "Creating tag $TAG..."
git tag "$TAG"
git push origin main
git push origin "$TAG"

echo "Done! Release $TAG will be built by GitHub Actions."
echo "Track progress: https://github.com/$(git remote get-url origin | sed 's/.*github.com[:\/]\(.*\)\.git/\1/')/actions"
