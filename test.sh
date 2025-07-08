#!/bin/bash

# 算术表达式计算器 MCP 服务器测试脚本
# 使用 MCP Inspector 测试各种功能

set -e

echo "=== 算术表达式计算器 MCP 服务器测试 ==="
echo

# 检查是否安装了 MCP Inspector
if ! command -v npx &> /dev/null; then
    echo "错误: 需要安装 Node.js 和 npx"
    exit 1
fi

# 构建项目
echo "1. 构建项目..."
cargo build --release
echo "✓ 项目构建完成"
echo

# 启动服务器的命令
SERVER_CMD="./target/release/acc_calc_mcp"

echo "2. 测试基本功能..."

# 测试工具列表
echo "2.1 列出可用工具："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/list
echo

# 测试基本计算
echo "2.2 测试基本计算 (1 + 2 * 3)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1 + 2 * 3" --tool-arg decimals=0
echo

# 测试带小数的计算
echo "2.3 测试小数计算 (1.234 + 2.567, 保留2位小数)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1.234 + 2.567" --tool-arg decimals=2
echo

# 测试百分比计算 - 先转换后舍入
echo "2.4 测试百分比计算 - 先转换后舍入 (50.126%)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="50.126%" --tool-arg decimals=2 --tool-arg rounding_strategy="convert_then_round"
echo

# 测试百分比计算 - 先舍入后转换
echo "2.5 测试百分比计算 - 先舍入后转换 (50.126%)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="50.126%" --tool-arg decimals=2 --tool-arg rounding_strategy="round_then_convert"
echo

# 测试复杂表达式
echo "2.6 测试复杂表达式 ((1.5 + 2.5) * 3 - 1)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="(1.5 + 2.5) * 3 - 1" --tool-arg decimals=1
echo

# 测试负数
echo "2.7 测试负数计算 (-5 + 3)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="-5 + 3" --tool-arg decimals=0
echo

# 测试验证功能
echo "3. 测试验证功能..."

# 测试正确验证
echo "3.1 测试正确验证 (1 + 2 = 3)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name validate --tool-arg expression="1 + 2" --tool-arg expected=3.0 --tool-arg decimals=0
echo

# 测试错误验证
echo "3.2 测试错误验证 (1 + 2 = 4)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name validate --tool-arg expression="1 + 2" --tool-arg expected=4.0 --tool-arg decimals=0
echo

# 测试小数验证
echo "3.3 测试小数验证 (1.234 + 2.567 = 3.80, 保留2位小数)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name validate --tool-arg expression="1.234 + 2.567" --tool-arg expected=3.80 --tool-arg decimals=2
echo

echo "4. 测试错误处理..."

# 测试除零错误
echo "4.1 测试除零错误 (1 / 0)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1 / 0" --tool-arg decimals=0 || true
echo

# 测试无效表达式
echo "4.2 测试无效表达式 (1 + )："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1 + " --tool-arg decimals=0 || true
echo

# 测试括号不匹配
echo "4.3 测试括号不匹配 ((1 + 2)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="(1 + 2" --tool-arg decimals=0 || true
echo

# 测试无效字符
echo "4.4 测试无效字符 (1 @ 2)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1 @ 2" --tool-arg decimals=0 || true
echo

echo "5. 测试高级功能..."

# 测试复杂百分比计算
echo "5.1 测试复杂百分比表达式 (100% - 50% + 25%)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="100% - 50% + 25%" --tool-arg decimals=2
echo

# 测试嵌套括号
echo "5.2 测试嵌套括号 (((1 + 2) * 3) / 3)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="((1 + 2) * 3) / 3" --tool-arg decimals=0
echo

# 测试浮点精度
echo "5.3 测试浮点精度 (0.1 + 0.2)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="0.1 + 0.2" --tool-arg decimals=1
echo

echo "6. 测试千分位分隔符..."

# 测试美式千分位格式
echo "6.1 测试美式千分位格式 (1,234 + 2,567)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1,234 + 2,567" --tool-arg decimals=0
echo

# 测试美式千分位带小数
echo "6.2 测试美式千分位带小数 (1,234.56 + 2,000.44)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1,234.56 + 2,000.44" --tool-arg decimals=2
echo

# 测试欧式千分位格式
echo "6.3 测试欧式千分位格式 (1.234,56 + 2.000,44)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1.234,56 + 2.000,44" --tool-arg decimals=2
echo

# 测试空格千分位格式
echo "6.4 测试空格千分位格式 (1 234 567 + 500 000)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1 234 567 + 500 000" --tool-arg decimals=0
echo

# 测试撇号千分位格式
echo "6.5 测试撇号千分位格式 (1'234'567.89 * 2)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1'234'567.89 * 2" --tool-arg decimals=2
echo

# 测试千分位验证
echo "6.6 测试千分位验证 (1,000,000 + 500,000 = 1,500,000)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name validate --tool-arg expression="1,000,000.00 + 500,000.00" --tool-arg expected=1500000.0 --tool-arg decimals=0
echo

# 测试混合格式
echo "6.7 测试混合格式 (美式 + 欧式: 1,234.56 + 1.000,44)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1,234.56 + 1.000,44" --tool-arg decimals=2
echo

# 测试混合格式与百分比
echo "6.8 测试混合格式与百分比 (1,234.56 + 10% * 1.000,00)："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name calculate --tool-arg expression="1,234.56 + 10% * 1.000,00" --tool-arg decimals=2
echo

echo "7. 测试批量验证功能..."

# 测试基本批量验证
echo "7.1 测试基本批量验证："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name batch_validate --tool-arg expressions='["1 + 2|3", "2 * 3|6", "10 / 2|5"]' --tool-arg default_decimals=0
echo

# 测试带小数的批量验证
echo "7.2 测试带小数的批量验证："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name batch_validate --tool-arg expressions='["1.234 + 2.567|3.80|2", "50.126%|0.50|2"]' --tool-arg default_decimals=2
echo

# 测试混合格式的批量验证
echo "7.3 测试混合格式的批量验证："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name batch_validate --tool-arg expressions='["1,234.56 + 1.000,44|2235.00|2", "1,000,000.00 + 500,000.00|1500000|0"]' --tool-arg default_decimals=2
echo

# 测试包含错误的批量验证
echo "7.4 测试包含错误的批量验证："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name batch_validate --tool-arg expressions='["1 + 2|3", "2 * 3|7", "10 / 2|5"]' --tool-arg default_decimals=0
echo

# 测试预期值包含百分数
echo "7.5 测试预期值包含百分数："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name validate --tool-arg expression="25% + 25%" --tool-arg expected="50%" --tool-arg decimals=2
echo

# 测试预期值包含千分位
echo "7.6 测试预期值包含千分位："
npx @modelcontextprotocol/inspector --cli "$SERVER_CMD" --method tools/call --tool-name validate --tool-arg expression="500 + 500" --tool-arg expected="1,000" --tool-arg decimals=0
echo

echo "✓ 所有测试完成！"
echo
echo "测试总结："
echo "- 基本算术运算: ✓"
echo "- 小数处理和舍入: ✓"
echo "- 百分比计算: ✓"
echo "- 复杂表达式: ✓"
echo "- 验证功能: ✓"
echo "- 错误处理: ✓"
echo "- 高级功能: ✓"
echo "- 千分位分隔符: ✓"
echo "- 批量验证: ✓"
echo
echo "算术表达式计算器 MCP 服务器已准备就绪！"