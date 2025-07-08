use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "算术表达式计算器 MCP 服务器")]
pub struct CommandArguments {
    /// 服务器启动后的欢迎消息
    #[arg(
        long,
        help = "Display startup message",
        default_value = "Calculator MCP Server is running"
    )]
    pub startup_message: String,
}