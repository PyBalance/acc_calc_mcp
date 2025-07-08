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
    /// 百分数处理策略（仅当表达式包含百分数时有效）：divide_by_100_then_round（先除以100后舍入）或 round_then_divide_by_100（先舍入后除以100），默认是 divide_by_100_then_round
    #[serde(default = "default_percent_rounding")]
    pub percent_rounding: String,
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
    /// 预期的结果值（支持百分数和千分位格式，如：50.5%, 1,234.56, 1.234,56）
    pub expected: String,
    /// 要保留的小数位数
    pub decimals: u32,
    /// 百分数处理策略（仅当表达式或预期值包含百分数时有效）：divide_by_100_then_round（先除以100后舍入）或 round_then_divide_by_100（先舍入后除以100), 默认是 divide_by_100_then_round
    #[serde(default = "default_percent_rounding")]
    pub percent_rounding: String,
}

#[mcp_tool(
    name = "batch_validate",
    title = "批量验证算术表达式",
    description = "同时验证多个算式的计算结果是否与预期值相符。支持批量处理多个表达式，提高验证效率。每个表达式都支持千分位分隔符（美式、欧式、空格、撇号格式）和完整的运算符集合。支持为每个表达式添加标记（如'流动资产合计'）以便识别错误的算式。",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, JsonSchema)]
pub struct BatchValidateTool {
    /// 要验证的表达式列表，格式为 "expression|expected" 或 "expression|expected|label"
    pub expressions: Vec<String>,
    /// 要保留的小数位数
    #[serde(default = "default_decimals")]
    pub decimals: u32,
    /// 百分数处理策略（仅当表达式包含百分数时有效）：divide_by_100_then_round（先除以100后舍入）或 round_then_divide_by_100（先舍入后除以100），默认是 divide_by_100_then_round
    #[serde(default = "default_percent_rounding")]
    pub percent_rounding: String,
}

fn default_decimals() -> u32 {
    2
}

fn default_percent_rounding() -> String {
    "divide_by_100_then_round".to_string()
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
                results.push(format!("行 {}: 格式错误 - 需要 'expression|expected' 格式", index + 1));
                all_passed = false;
                continue;
            }
            
            let expression = parts[0].trim();
            
            let label = if parts.len() > 2 {
                parts[2].trim().to_string()
            } else {
                String::new()
            };
            
            let label_prefix = if label.is_empty() {
                String::new()
            } else {
                format!("[{}] ", label)
            };
            
            let strategy = match parse_percent_rounding(&params.percent_rounding) {
                Ok(s) => s,
                Err(_) => {
                    results.push(format!("行 {}: {}无效的百分数处理策略 '{}'", index + 1, label_prefix, params.percent_rounding));
                    all_passed = false;
                    continue;
                }
            };
            
            let expected = match parse_expected_value(parts[1].trim(), params.decimals, strategy) {
                Ok(val) => val,
                Err(_) => {
                    results.push(format!("行 {}: {}无效的预期值 '{}'", index + 1, label_prefix, parts[1]));
                    all_passed = false;
                    continue;
                }
            };
            
            let is_valid = validate(expression, expected, params.decimals, strategy);
            
            if is_valid {
                results.push(format!("行 {}: {}{} = {} (通过)", index + 1, label_prefix, expression, expected));
            } else {
                // 计算实际值以便显示差异
                match calculate(expression, params.decimals, strategy) {
                    Ok(actual) => {
                        results.push(format!("行 {}: {}{} ≠ {} (实际: {})", index + 1, label_prefix, expression, expected, actual));
                    }
                    Err(e) => {
                        results.push(format!("行 {}: {}{} - 计算错误: {:?}", index + 1, label_prefix, expression, e));
                    }
                }
                all_passed = false;
            }
        }
        
        let summary = if all_passed {
            format!("批量验证完成！所有 {} 个表达式均通过验证", params.expressions.len())
        } else {
            let passed_count = results.iter().filter(|r| r.contains("(通过)")).count();
            let total_count = params.expressions.len();
            format!("批量验证完成！{}/{} 个表达式通过验证", passed_count, total_count)
        };
        
        let mut output = vec![summary, "".to_string()];
        output.extend(results);
        
        Ok(CallToolResult::text_content(vec![TextContent::from(
            output.join("\n")
        )]))
    }
}

fn parse_percent_rounding(strategy: &str) -> Result<PercentRounding, CallToolError> {
    match strategy {
        "divide_by_100_then_round" => Ok(PercentRounding::DivideBy100ThenRound),
        "round_then_divide_by_100" => Ok(PercentRounding::RoundThenDivideBy100),
        _ => Err(CallToolError::new(crate::error::ServiceError::InvalidExpression(
            format!("无效的百分数处理策略: {}，支持的策略：divide_by_100_then_round, round_then_divide_by_100", strategy)
        ))),
    }
}

fn parse_expected_value(expected_str: &str, decimals: u32, strategy: PercentRounding) -> Result<f64, CallToolError> {
    // 使用和计算器相同的逻辑来解析预期值
    let dummy_expr = expected_str.trim();
    
    // 如果包含百分号，需要按照策略处理
    if dummy_expr.contains('%') {
        // 创建一个简单的表达式来利用现有的计算逻辑
        let calc_result = calculate(dummy_expr, decimals, strategy)
            .map_err(|e| CallToolError::new(crate::error::ServiceError::from(e)))?;
        Ok(calc_result)
    } else {
        // 不包含百分号，使用现有的数字解析逻辑
        let mut chars = dummy_expr.chars().peekable();
        let num_str = consume_number_for_expected(&mut chars);
        
        if !num_str.is_empty() {
            let parsed = num_str.parse::<f64>()
                .map_err(|_| CallToolError::new(crate::error::ServiceError::InvalidExpression(
                    format!("无法解析预期值: {}", expected_str)
                )))?;
            Ok(parsed)
        } else {
            Err(CallToolError::new(crate::error::ServiceError::InvalidExpression(
                format!("无效的预期值格式: {}", expected_str)
            )))
        }
    }
}

// 辅助函数：为预期值解析数字（复用计算器的逻辑）
fn consume_number_for_expected(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    use crate::tools::calculator::normalize_number;
    
    let mut num_str = String::new();
    let mut has_digit = false;
    
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            has_digit = true;
            chars.next();
        } else if (c == '.' || c == ',' || c == ' ' || c == '\'' || c == '-') && has_digit {
            // 支持负数和千分位分隔符
            num_str.push(c);
            chars.next();
        } else if c == '-' && !has_digit {
            // 负号在开头
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }
    
    if has_digit {
        normalize_number(&num_str)
    } else {
        num_str
    }
}

impl CalculateTool {
    pub async fn run_tool(
        params: Self,
        _context: &(),
    ) -> Result<CallToolResult, CallToolError> {
        let strategy = parse_percent_rounding(&params.percent_rounding)?;
        
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
        let strategy = parse_percent_rounding(&params.percent_rounding)?;
        
        // 解析预期值，支持百分数和千分位
        let expected_value = parse_expected_value(&params.expected, params.decimals, strategy)?;
        
        let is_valid = validate(&params.expression, expected_value, params.decimals, strategy);
        
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!(
                "表达式: {}\n预期值: {} (解析为: {})\n验证结果: {}",
                params.expression,
                params.expected,
                expected_value,
                if is_valid { "通过" } else { "失败" }
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