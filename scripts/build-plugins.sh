#!/bin/bash
set -e

# Build all local plugins for development
# Usage: ./scripts/build-plugins.sh [plugin-name]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔨 Building plugins..."
echo ""

# Function to build a single plugin
build_plugin() {
    local plugin_name=$1
    local plugin_dir="$REPO_ROOT/plugins/$plugin_name"
    
    if [ ! -d "$plugin_dir" ]; then
        echo "❌ Plugin not found: $plugin_name"
        return 1
    fi
    
    echo "📦 Building $plugin_name..."
    
    # Build the plugin
    cargo component build \
        --manifest-path="$plugin_dir/Cargo.toml" \
        --release
    
    # Find the built WASM (convert plugin-name to plugin_name for the binary)
    local wasm_name=$(echo "$plugin_name" | tr '-' '_')_plugin
    local wasm_path="$REPO_ROOT/target/wasm32-wasip1/release/${wasm_name}.wasm"
    
    if [ ! -f "$wasm_path" ]; then
        echo "❌ WASM not found at: $wasm_path"
        return 1
    fi
    
    # Copy to expected location
    cp "$wasm_path" "$plugin_dir/plugin.wasm"
    
    local size=$(du -h "$plugin_dir/plugin.wasm" | cut -f1)
    echo "✓ Built $plugin_name ($size)"
    echo ""
}

# If specific plugin provided, build only that one
if [ -n "$1" ]; then
    build_plugin "$1"
    exit 0
fi

# Otherwise, build all plugins
for plugin_dir in "$REPO_ROOT"/plugins/*/; do
    if [ -d "$plugin_dir" ]; then
        plugin_name=$(basename "$plugin_dir")
        build_plugin "$plugin_name" || true
    fi
done

echo "✅ All plugins built successfully!"
