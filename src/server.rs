use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
    LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::{mcp_server::server_runtime, McpServer, StdioTransport, TransportOptions};

use crate::{cli::CommandArguments, error::ServiceResult, handler::CalculatorHandler};

pub fn server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: "acc-calc-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("算术表达式计算器 MCP 服务器: 支持自定义舍入规则的高精度计算工具".to_string()),
        },
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            completions: None,
        },
        instructions: None,
        meta: None,
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    }
}

pub async fn start_server(args: CommandArguments) -> ServiceResult<()> {
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| crate::error::ServiceError::Sdk(e.to_string()))?;
    
    let handler = CalculatorHandler::new(&args)?;
    let server = server_runtime::create_server(server_details(), transport, handler);
    
    server.start().await.map_err(|e| crate::error::ServiceError::Sdk(e.to_string()))?;
    
    Ok(())
}