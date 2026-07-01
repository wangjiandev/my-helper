#!/usr/bin/env bash
# 双击运行：处理本工具所在目录下的所有 PDF 发票，生成 output.pdf。
set -e
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
# 清除网络下载带来的隔离属性，避免加载 libpdfium 时被 macOS 拦截
xattr -dr com.apple.quarantine . 2>/dev/null || true
chmod +x ./invoice-printer 2>/dev/null || true
./invoice-printer --dir . --out ./output.pdf
echo
echo "===== 完成！已生成 output.pdf ====="
echo "按回车键关闭窗口..."
read -r
