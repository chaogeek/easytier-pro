#!/bin/bash
# 生成版本信息文件（用于构建时注入版本号）
# 用法：bash scripts/generate-version.sh

set -euo pipefail

VERSION_FILE="$(cd "$(dirname "$0")/.." && pwd)/VERSION"
VERSION=$(tr -d '[:space:]' < "$VERSION_FILE")

echo "// 此文件由 scripts/generate-version.sh 自动生成，请勿手动修改"
echo "// 版本号来源：VERSION 文件"
echo ""
echo "pub const APP_VERSION: &str = \"${VERSION}\";"

echo "版本号：${VERSION}"
