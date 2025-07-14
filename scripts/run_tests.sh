#!/bin/bash

# 测试运行脚本
# 用法: ./scripts/run_tests.sh [test_name]

set -e

echo "🧪 运行 Rust System Tools 测试"
echo "================================"

# 检查是否提供了特定的测试名称
if [ $# -eq 0 ]; then
    echo "📋 运行所有测试..."
    cargo test
else
    echo "🎯 运行特定测试: $1"
    cargo test --test wim_parser_test "$1"
fi

echo ""
echo "✅ 测试完成！"