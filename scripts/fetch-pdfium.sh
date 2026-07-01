#!/usr/bin/env bash
# 下载 libpdfium 到项目 lib/ 目录。
# 用法：bash scripts/fetch-pdfium.sh
set -euo pipefail

DIR="$(cd "$(dirname "$0")/.." && pwd)"
LIB_DIR="$DIR/lib"
mkdir -p "$LIB_DIR"

# 自动判断平台
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS-$ARCH" in
    Darwin-arm64)   ASSET="pdfium-mac-arm64.tgz" ;;
    Darwin-x86_64)  ASSET="pdfium-mac-x64.tgz" ;;
    Linux-x86_64)   ASSET="pdfium-linux-x64.tgz" ;;
    Linux-aarch64)  ASSET="pdfium-linux-arm64.tgz" ;;
    MINGW*-x86_64|MSYS*-x86_64) ASSET="pdfium-win-x64.tgz" ;;
    *) echo "不支持的平台: $OS-$ARCH"; exit 1 ;;
esac

# 取最新 chromium 版本
TAG="$(curl -sL https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p' | head -1)"
echo "下载 pdfium $TAG ($ASSET)..."

URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${TAG}/${ASSET}"
TMP="$(mktemp -d)"
curl -sL -o "$TMP/pdfium.tgz" "$URL"
tar -xzf "$TMP/pdfium.tgz" -C "$TMP"

# 定位动态库并拷贝
DYLIB="$(find "$TMP" -name "*.dylib" -o -name "*.so*" -o -name "*.dll" | head -1)"
if [ -z "$DYLIB" ]; then
    echo "未在压缩包中找到动态库"; exit 1
fi
cp "$DYLIB" "$LIB_DIR/"
# 移除 macOS 隔离属性（避免加载报错）
if [ "$OS" = "Darwin" ]; then
    xattr -dr com.apple.quarantine "$LIB_DIR" 2>/dev/null || true
fi
echo "完成: $(ls "$LIB_DIR")"
