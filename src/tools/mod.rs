pub mod calculator;

use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};

pub use calculator::{calculate, validate, PercentRounding};
pub use rust_mcp_sdk::tool_box;

#[mcp_tool(
    name = "calculate",
    title = "计算算术表达式",
    description = "给定任何符合规范的算式（运算符支持：加、减、乘、除、括号和百分号），支持千分位分隔符（美式: 1,234.56, 欧式: 1.234,56, 空格: 1 234.56, 撇号: 1'234.56）。运算特点：1. 所有数字在参与运算前，根据指定小数位数进行四舍五入；2. 计算结果也需要进行最终的四舍五入；3. 计算过程不进行四舍五入。",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, JsonSchema)]
pub struct CalculateTool {
    /// 要计算的算术表达式（运算符支持：加、减、乘、除、括号和百分号），支持千分位分隔符（美式: 1,234.56, 欧式: 1.234,56, 空格: 1 234.56, 撇号: 1'234.56）
    pub expression: String,
    /// 计算前和结果要保留的小数位数
    pub decimals: u32,
    /// 百分比舍入策略：convert_then_round（先转换后舍入）或 round_then_convert（先舍入后转换）
    #[serde(default = "default_rounding_strategy")]
    pub rounding_strategy: String,
}

#[mcp_tool(
    name = "validate",
    title = "验证算术表达式",
    description = "验证给定算式的计算结果是否与预期值相符（运算符支持：加、减、乘、除、括号和百分号），支持千分位分隔符（美式、欧式、空格、撇号格式）。验证过程遵循与计算工具相同的运算规则：1. 所有数字在参与运算前，根据指定小数位数进行四舍五入；2. 计算结果也需要进行最终的四舍五入；3. 计算过程不进行四舍五入。",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, JsonSchema)]
pub struct ValidateTool {
    /// 要验证的算术表达式（支持千分位分隔符：美式、欧式、空格、撇号格式）
    pub expression: String,
    /// 预期的结果值
    pub expected: f64,
    /// 要保留的小数位数
    pub decimals: u32,
    /// 百分比舍入策略：convert_then_round（先转换后舍入）或 round_then_convert（先舍入后转换）
    #[serde(default = "default_rounding_strategy")]
    pub rounding_strategy: String,
}

#[mcp_tool(
    name = "batch_validate",
    title = "批量验证算术表达式",
    description = "同时验证多个算式的计算结果是否与预期值相符。支持批量处理多个表达式，提高验证效率。每个表达式都支持千分位分隔符（美式、欧式、空格、撇号格式）和完整的运算符集合。",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, JsonSchema)]
pub struct BatchValidateTool {
    /// 要验证的表达式列表，格式为 "expression|expected" 或 "expression|expected|decimals" 或 "expression|expected|decimals|rounding_strategy"
    pub expressions: Vec<String>,
    /// 默认要保留的小数位数（如果表达式中未指定）
    #[serde(default = "default_decimals")]
    pub default_decimals: u32,
    /// 默认百分比舍入策略（如果表达式中未指定）
    #[serde(default = "default_rounding_strategy")]
    pub default_rounding_strategy: String,
}

fn default_decimals() -> u32 {
    2
}

fn default_rounding_strategy() -> String {
    "convert_then_round".to_string()
}

impl BatchValidateTool {
    pub async fn run_tool(
        params: Self,
        _context: &(),
    ) -> Result<CallToolResult, CallToolError> {
        let mut results = Vec::new();
        let mut all_passed = true;
        
        for (index, expr_line) in params.expressions.iter().enumerate() {
            let parts: Vec<&str> = expr_line.split('|').collect();
            
            if parts.len() < 2 {
                results.push(format!("❌ 行 {}: 格式错误 - 需要 'expression|expected' 格式", index + 1));
                all_passed = false;
                continue;
            }
            
            let expression = parts[0].trim();
            let expected = match parts[1].trim().parse::<f64>() {
                Ok(val) => val,
                Err(_) => {
                    results.push(format!("❌ 行 {}: 无效的预期值 '{}'", index + 1, parts[1]));
                    all_passed = false;
                    continue;
                }
            };
            
            let decimals = if parts.len() > 2 {
                match parts[2].trim().parse::<u32>() {
                    Ok(val) => val,
                    Err(_) => {
                        results.push(format!("❌ 行 {}: 无效的小数位数 '{}'", index + 1, parts[2]));
                        all_passed = false;
                        continue;
                    }
                }
            } else {
                params.default_decimals
            };
            
            let rounding_strategy = if parts.len() > 3 {
                parts[3].trim().to_string()
            } else {
                params.default_rounding_strategy.clone()
            };
            
            let strategy = match parse_rounding_strategy(&rounding_strategy) {
                Ok(s) => s,
                Err(_) => {
                    results.push(format!("❌ 行 {}: 无效的舍入策略 '{}'", index + 1, rounding_strategy));
                    all_passed = false;
                    continue;
                }
            };
            
            let is_valid = validate(expression, expected, decimals, strategy);
            
            if is_valid {
                results.push(format!("✅ 行 {}: {} = {} (通过)", index + 1, expression, expected));
            } else {
                // 计算实际值以便显示差异
                match calculate(expression, decimals, strategy) {
                    Ok(actual) => {
                        results.push(format!("❌ 行 {}: {} ≠ {} (实际: {})", index + 1, expression, expected, actual));
                    }
                    Err(e) => {
                        results.push(format!("❌ 行 {}: {} - 计算错误: {:?}", index + 1, expression, e));
                    }
                }
                all_passed = false;
            }
        }
        
        let summary = if all_passed {
            format!("🎉 批量验证完成！所有 {} 个表达式均通过验证", params.expressions.len())
        } else {
            let passed_count = results.iter().filter(|r| r.starts_with("✅")).count();
            let total_count = params.expressions.len();
            format!("⚠️  批量验证完成！{}/{} 个表达式通过验证", passed_count, total_count)
        };
        
        let mut output = vec![summary, "".to_string()];
        output.extend(results);
        
        Ok(CallToolResult::text_content(vec![TextContent::from(
            output.join("\n")
        )]))
    }
}

fn parse_rounding_strategy(strategy: &str) -> Result<PercentRounding, CallToolError> {
    match strategy {
        "convert_then_round" => Ok(PercentRounding::ConvertThenRound),
        "round_then_convert" => Ok(PercentRounding::RoundThenConvert),
        _ => Err(CallToolError::new(crate::error::ServiceError::InvalidExpression(
            format!("无效的舍入策略: {}，支持的策略：convert_then_round, round_then_convert", strategy)
        ))),
    }
}

impl CalculateTool {
    pub async fn run_tool(
        params: Self,
        _context: &(),
    ) -> Result<CallToolResult, CallToolError> {
        let strategy = parse_rounding_strategy(&params.rounding_strategy)?;
        
        let result = calculate(&params.expression, params.decimals, strategy)
            .map_err(|e| CallToolError::new(crate::error::ServiceError::from(e)))?;
        
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("表达式: {}\n结果: {}", params.expression, result)
        )]))
    }
}

impl ValidateTool {
    pub async fn run_tool(
        params: Self,
        _context: &(),
    ) -> Result<CallToolResult, CallToolError> {
        let strategy = parse_rounding_strategy(&params.rounding_strategy)?;
        
        let is_valid = validate(&params.expression, params.expected, params.decimals, strategy);
        
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!(
                "表达式: {}\n预期值: {}\n验证结果: {}",
                params.expression,
                params.expected,
                if is_valid { "✓ 通过" } else { "✗ 失败" }
            )
        )]))
    }
}

tool_box!(
    CalculatorTools,
    [
        CalculateTool,
        ValidateTool,
        BatchValidateTool
    ]
);