use std::cmp::Ordering;

use crate::cli::CommandArguments;
use crate::error::ServiceResult;
use crate::tools::*;
use async_trait::async_trait;
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError, CallToolRequest, CallToolResult, InitializeRequest,
    InitializeResult, ListToolsRequest, ListToolsResult, RpcError,
};
use rust_mcp_sdk::McpServer;

pub struct CalculatorHandler {
    startup_message: String,
}

impl CalculatorHandler {
    pub fn new(args: &CommandArguments) -> ServiceResult<Self> {
        Ok(Self {
            startup_message: args.startup_message.clone(),
        })
    }
}

#[async_trait]
impl ServerHandler for CalculatorHandler {
    async fn on_server_started(&self, runtime: &dyn McpServer) {
        let _ = runtime.stderr_message(self.startup_message.clone()).await;
    }

    async fn on_initialized(&self, _: &dyn McpServer) {}

    async fn handle_list_tools_request(
        &self,
        _: ListToolsRequest,
        _: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: CalculatorTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_initialize_request(
        &self,
        initialize_request: InitializeRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<InitializeResult, RpcError> {
        runtime
            .set_client_details(initialize_request.params.clone())
            .map_err(|err| RpcError::internal_error().with_message(format!("{err}")))?;

        let mut server_info = runtime.server_info().to_owned();
        if server_info
            .protocol_version
            .cmp(&initialize_request.params.protocol_version)
            == Ordering::Greater
        {
            server_info.protocol_version = initialize_request.params.protocol_version;
        }
        Ok(server_info)
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _: &dyn McpServer,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_params: CalculatorTools =
            CalculatorTools::try_from(request.params).map_err(CallToolError::new)?;

        match tool_params {
            CalculatorTools::CalculateTool(params) => {
                CalculateTool::run_tool(params, &()).await
            }
            CalculatorTools::ValidateTool(params) => {
                ValidateTool::run_tool(params, &()).await
            }
            CalculatorTools::BatchValidateTool(params) => {
                BatchValidateTool::run_tool(params, &()).await
            }
        }
    }
}