# 算术表达式计算器 MCP 服务器

一个基于 MCP (Model Context Protocol) 的算术表达式计算器服务器，支持自定义舍入规则和百分比处理。

## 功能特性

- 🧮 **完整的算术运算**: 支持加、减、乘、除、括号和百分号
- 🎯 **精确的舍入控制**: 支持指定小数位数的四舍五入
- 📊 **灵活的百分比处理**: 两种舍入策略（先转换后舍入 vs 先舍入后转换）
- 🌍 **多格式千分位支持**: 美式 (1,234.56)、欧式 (1.234,56)、空格 (1 234.56)、撇号 (1'234.56)
- ✅ **表达式验证**: 验证计算结果是否与预期值相符
- 🔧 **标准 MCP 协议**: 与任何支持 MCP 的客户端兼容

## 安装依赖

确保系统已安装：
- Rust 1.70+
- Node.js 18+ (用于测试)

## 构建和运行

```bash
# 克隆项目
git clone <your-repo-url>
cd acc_calc_mcp

# 构建项目
cargo build --release

# 运行测试
cargo test

# 启动 MCP 服务器
cargo run
```

## MCP 工具

### 1. calculate 工具

计算算术表达式并返回结果。

**参数**:
- `expression` (string): 要计算的算术表达式（支持多种千分位格式）
- `decimals` (number): 要保留的小数位数
- `rounding_strategy` (string, 可选): 百分比舍入策略
  - `"convert_then_round"` (默认): 先转换为小数后舍入
  - `"round_then_convert"`: 先舍入后转换为小数

**示例**:
```bash
# 基本计算
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name calculate --tool-arg expression="1 + 2 * 3" --tool-arg decimals=0

# 小数计算
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name calculate --tool-arg expression="1.234 + 2.567" --tool-arg decimals=2

# 百分比计算
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name calculate --tool-arg expression="50.126%" --tool-arg decimals=2 --tool-arg rounding_strategy="convert_then_round"

# 千分位分隔符计算
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name calculate --tool-arg expression="1,234.56 + 2,000.44" --tool-arg decimals=2

# 欧式格式
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name calculate --tool-arg expression="1.234,56 + 2.000,44" --tool-arg decimals=2
```

### 2. validate 工具

验证算术表达式的计算结果是否与预期值相符。

**参数**:
- `expression` (string): 要验证的算术表达式（支持多种千分位格式）
- `expected` (number): 预期的结果值
- `decimals` (number): 要保留的小数位数
- `rounding_strategy` (string, 可选): 百分比舍入策略

**示例**:
```bash
# 验证计算结果
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name validate --tool-arg expression="1 + 2" --tool-arg expected=3.0 --tool-arg decimals=0
```

### 3. batch_validate 工具

批量验证多个算术表达式的计算结果，提高验证效率。

**参数**:
- `expressions` (array): 表达式列表，每项格式为 `"expression|expected"` 或 `"expression|expected|decimals"` 或 `"expression|expected|decimals|rounding_strategy"`
- `default_decimals` (number, 可选): 默认小数位数，默认为2
- `default_rounding_strategy` (string, 可选): 默认百分比舍入策略

**示例**:
```bash
# 批量验证基本表达式
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name batch_validate --tool-arg expressions='["1 + 2|3", "2 * 3|6", "10 / 2|5"]' --tool-arg default_decimals=0

# 批量验证带小数的表达式
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name batch_validate --tool-arg expressions='["1.234 + 2.567|3.80|2", "50.126%|0.50|2"]'

# 批量验证混合格式
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name batch_validate --tool-arg expressions='["1,234.56 + 1.000,44|2235.00|2", "1,000,000.00 + 500,000.00|1500000|0"]'
```

## 快速测试

运行包含的测试脚本来验证所有功能：

```bash
./test.sh
```

测试脚本将验证：
- 基本算术运算
- 小数处理和舍入
- 百分比计算（两种策略）
- 千分位分隔符（四种格式）
- 复杂表达式
- 单个验证功能
- 批量验证功能
- 错误处理
- 高级功能

## 使用 MCP Inspector 测试

### 列出可用工具
```bash
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/list
```

### 调用 calculate 工具
```bash
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name calculate --tool-arg expression="(1.5 + 2.5) * 3 - 1" --tool-arg decimals=1
```

### 调用 validate 工具
```bash
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name validate --tool-arg expression="1.234 + 2.567" --tool-arg expected=3.80 --tool-arg decimals=2
```

### 调用 batch_validate 工具
```bash
npx @modelcontextprotocol/inspector --cli ./target/release/acc_calc_mcp --method tools/call --tool-name batch_validate --tool-arg expressions='["1 + 2|3", "2 * 3|6", "10 / 2|5"]' --tool-arg default_decimals=0
```

## 算法特点

### 舍入规则
1. **输入舍入**: 所有数字在参与运算前，根据指定小数位数进行四舍五入
2. **计算过程**: 使用完整精度进行计算
3. **结果舍入**: 最终结果按指定小数位数四舍五入

### 百分比处理策略

#### convert_then_round (先转换后舍入)
```
50.126% → 0.50126 → 0.50 (保留2位小数)
```

#### round_then_convert (先舍入后转换)
```
50.126% → 50.13 → 0.5013 (保留2位小数) → 0.50 (最终舍入)
```

## 支持的运算符

- `+` 加法
- `-` 减法和负号
- `*` 乘法
- `/` 除法
- `()` 括号（支持嵌套）
- `%` 百分号

## 支持的数字格式

### 千分位分隔符
- **美式格式**: `1,234.56` (逗号分隔千位，点号小数点)
- **欧式格式**: `1.234,56` (点号分隔千位，逗号小数点)
- **空格格式**: `1 234.56` (空格分隔千位)
- **撇号格式**: `1'234.56` (撇号分隔千位)

### 格式检测规则
- 自动检测数字格式，无需指定
- 支持大数字：`1,000,000` 或 `1.000.000,00`
- 智能区分千分位分隔符和小数点
- 混合格式在同一表达式中使用

## 错误处理

服务器会优雅地处理以下错误：
- 除零错误
- 无效表达式
- 括号不匹配
- 无效字符
- 表达式意外结束

## 项目结构

```
acc_calc_mcp/
├── src/
│   ├── main.rs          # 主入口
│   ├── cli.rs           # 命令行参数
│   ├── server.rs        # MCP 服务器设置
│   ├── handler.rs       # 请求处理器
│   ├── error.rs         # 错误类型
│   └── tools/
│       ├── mod.rs       # 工具模块
│       └── calculator.rs # 计算器核心实现
├── test.sh              # 测试脚本
├── Cargo.toml           # 依赖配置
└── README.md            # 项目文档
```

## 开发

### 添加新工具

1. 在 `src/tools/mod.rs` 中定义新的工具结构体
2. 使用 `#[mcp_tool]` 属性宏标注
3. 实现 `run_tool` 方法
4. 将工具添加到 `tool_box!` 宏中

### 运行单元测试

```bash
cargo test
```

### 构建发布版本

```bash
cargo build --release
```

## 许可证

本项目基于 MIT 许可证开源。