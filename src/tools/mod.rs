pub mod calculator;

use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};

pub use calculator::{calculate, validate, PercentRounding};
pub use rust_mcp_sdk::tool_box;

#[mcp_tool(
    name = "calculate",
    title = "计算算术表达式",
    description = "根据自定义规则计算一个字符串形式的算术表达式。支持加减乘除、括号和百分号，所有数字按指定小数位数四舍五入。",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, JsonSchema)]
pub struct CalculateTool {
    /// 要计算的算术表达式
    pub expression: String,
    /// 要保留的小数位数
    pub decimals: u32,
    /// 百分比舍入策略：convert_then_round（先转换后舍入）或 round_then_convert（先舍入后转换）
    #[serde(default = "default_rounding_strategy")]
    pub rounding_strategy: String,
}

#[mcp_tool(
    name = "validate",
    title = "验证算术表达式",
    description = "验证一个算式的计算结果是否与预期值相符。",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, JsonSchema)]
pub struct ValidateTool {
    /// 要验证的算术表达式
    pub expression: String,
    /// 预期的结果值
    pub expected: f64,
    /// 要保留的小数位数
    pub decimals: u32,
    /// 百分比舍入策略：convert_then_round（先转换后舍入）或 round_then_convert（先舍入后转换）
    #[serde(default = "default_rounding_strategy")]
    pub rounding_strategy: String,
}

fn default_rounding_strategy() -> String {
    "convert_then_round".to_string()
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
        ValidateTool
    ]
);