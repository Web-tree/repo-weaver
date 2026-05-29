#!/bin/bash
set -e

# Release a plugin with proper semver tagging
# Usage: ./scripts/release-plugin.sh <plugin-name> <version>
# Example: ./scripts/release-plugin.sh npm-script 1.0.0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ $# -ne 2 ]; then
    echo "Usage: $0 <plugin-name> <version>"
    echo "Example: $0 npm-script 1.0.0"
    exit 1
fi

PLUGIN_NAME=$1
VERSION=$2
TAG="${PLUGIN_NAME}-v${VERSION}"

echo "🚀 Preparing to release $PLUGIN_NAME v$VERSION"
echo ""

# Verify plugin exists
if [ ! -d "$REPO_ROOT/plugins/$PLUGIN_NAME" ]; then
    echo "❌ Plugin not found: plugins/$PLUGIN_NAME"
    exit 1
fi

# Check if on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "⚠️  Warning: You are on branch '$CURRENT_BRANCH', not 'main'"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Update Cargo.toml version
echo "📝 Updating version in Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" "$REPO_ROOT/plugins/$PLUGIN_NAME/Cargo.toml"
rm "$REPO_ROOT/plugins/$PLUGIN_NAME/Cargo.toml.bak"

# Verify version was updated
CARGO_VERSION=$(grep '^version' "$REPO_ROOT/plugins/$PLUGIN_NAME/Cargo.toml" | cut -d'"' -f2)
if [ "$CARGO_VERSION" != "$VERSION" ]; then
    echo "❌ Failed to update version in Cargo.toml"
    exit 1
fi
echo "✓ Version updated to $VERSION"
echo ""

# Show git diff
echo "📋 Changes to be committed:"
git diff --color "plugins/$PLUGIN_NAME/Cargo.toml"
echo ""

# Confirm
read -p "Commit and tag? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted. Changes not committed."
    git checkout "plugins/$PLUGIN_NAME/Cargo.toml"
    exit 1
fi

# Commit
git add "plugins/$PLUGIN_NAME/Cargo.toml"
git commit -m "chore($PLUGIN_NAME): bump version to $VERSION"

# Create tag
git tag -a "$TAG" -m "Release $PLUGIN_NAME v$VERSION"

echo ""
echo "✅ Local changes committed and tagged"
echo ""
echo "📤 To push and trigger release:"
echo "   git push origin main"
echo "   git push origin $TAG"
echo ""
echo "Or push both at once:"
echo "   git push origin main --tags"
