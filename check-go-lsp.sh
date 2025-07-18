#!/bin/bash

# Go LSP Setup Checker for ls-pretty
# This script helps verify that Go language server is properly configured

echo "🔍 Checking Go LSP Setup for ls-pretty..."
echo "================================================"

# Check if Go is installed
echo -n "1. Checking Go installation... "
if command -v go &> /dev/null; then
    echo "✅ Found: $(go version)"
else
    echo "❌ Go not found - please install Go first"
    exit 1
fi

# Check GOPATH and GOROOT
echo "2. Go environment:"
echo "   GOPATH: $(go env GOPATH)"
echo "   GOROOT: $(go env GOROOT)"

# Check if gopls is installed
echo -n "3. Checking gopls installation... "
if command -v gopls &> /dev/null; then
    echo "✅ Found: $(gopls version 2>&1 | head -1)"
else
    echo "❌ gopls not found"
    echo "   To install: go install golang.org/x/tools/gopls@latest"
    echo "   Then add $(go env GOPATH)/bin to your PATH"
    exit 1
fi

# Check if GOPATH/bin is in PATH
echo -n "4. Checking PATH configuration... "
GOPATH_BIN="$(go env GOPATH)/bin"
if echo "$PATH" | grep -q "$GOPATH_BIN"; then
    echo "✅ GOPATH/bin is in PATH"
else
    echo "⚠️  GOPATH/bin not found in PATH"
    echo "   Add this to your shell profile:"
    echo "   export PATH=\$PATH:$(go env GOPATH)/bin"
fi

# Test gopls functionality
echo -n "5. Testing gopls server... "
cat > test_lsp.go << 'EOF'
package main

import "fmt"

func main() {
    fmt.
EOF

# Try to start gopls and see if it responds
if timeout 5s gopls serve < /dev/null &> /dev/null; then
    echo "✅ gopls server starts successfully"
else
    echo "⚠️  gopls server may have issues"
fi

rm -f test_lsp.go

echo ""
echo "📋 Summary:"
echo "================================================"

if command -v gopls &> /dev/null && echo "$PATH" | grep -q "$(go env GOPATH)/bin"; then
    echo "✅ Go LSP is properly configured!"
    echo ""
    echo "🚀 To test in ls-pretty:"
    echo "   1. Run: cargo run ."
    echo "   2. Open test.go file"
    echo "   3. Press Enter to view, then Ctrl+E to edit"
    echo "   4. Look for '🟢 LSP' in the header"
    echo "   5. Type 'fmt.' and press Ctrl+Space"
    echo "   6. You should see autocomplete suggestions"
else
    echo "❌ Go LSP setup incomplete"
    echo ""
    echo "🔧 Required steps:"
    if ! command -v gopls &> /dev/null; then
        echo "   - Install gopls: go install golang.org/x/tools/gopls@latest"
    fi
    if ! echo "$PATH" | grep -q "$(go env GOPATH)/bin"; then
        echo "   - Add to PATH: export PATH=\$PATH:$(go env GOPATH)/bin"
    fi
fi

echo ""
echo "ℹ️  For more info, see the Go LSP section in README.md"
