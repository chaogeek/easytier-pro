#!/bin/bash
# 下载 easytier-core 和 easytier-cli 二进制文件
# 用于构建时嵌入到 App Bundle 中
# 用法：bash scripts/download-easycore.sh [版本号]

set -euo pipefail

VERSION="${1:-2.6.4}"
ARCH=$(uname -m)

# 架构映射：arm64 → aarch64
if [ "$ARCH" = "arm64" ]; then
  ARCH_URL="aarch64"
else
  ARCH_URL="x86_64"
fi

CACHE_DIR="$HOME/Library/Caches/com.easytier.manager/easytier-binaries"
HELPERS_DIR="$(cd "$(dirname "$0")/.." && pwd)/Helpers"

mkdir -p "$CACHE_DIR" "$HELPERS_DIR"

ZIP_NAME="easytier-macos-${ARCH_URL}-v${VERSION}.zip"
DOWNLOAD_URL="https://github.com/EasyTier/EasyTier/releases/download/v${VERSION}/${ZIP_NAME}"

echo "下载 easytier-core v${VERSION}（架构: ${ARCH_URL}）..."
echo "URL: $DOWNLOAD_URL"

# 检查缓存
if [ -f "$CACHE_DIR/$ZIP_NAME" ]; then
  echo "使用缓存: $CACHE_DIR/$ZIP_NAME"
else
  curl -fSL --connect-timeout 30 --max-time 300 \
    -o "$CACHE_DIR/$ZIP_NAME" \
    "$DOWNLOAD_URL"
fi

# 解压到 Helpers 目录
echo "解压到 $HELPERS_DIR ..."
unzip -o "$CACHE_DIR/$ZIP_NAME" -d "$HELPERS_DIR"

# 设置可执行权限
chmod +x "$HELPERS_DIR/easytier-core" "$HELPERS_DIR/easytier-cli" 2>/dev/null || true

echo "完成！"
echo "  easytier-core: $(file "$HELPERS_DIR/easytier-core" 2>/dev/null || echo '未找到')"
echo "  easytier-cli:  $(file "$HELPERS_DIR/easytier-cli" 2>/dev/null || echo '未找到')"
