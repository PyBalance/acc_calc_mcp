use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("计算错误: {0}")]
    CalculationError(String),
    
    #[error("无效表达式: {0}")]
    InvalidExpression(String),
    
    #[error("除零错误")]
    DivisionByZero,
    
    #[error("无效字符: {0}")]
    InvalidCharacter(char),
    
    #[error("括号不匹配")]
    MismatchedParens,
    
    #[error("表达式意外结束")]
    UnexpectedEndOfExpression,
    
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("MCP SDK 错误: {0}")]
    Sdk(String),
    
    #[error("通用错误: {0}")]
    Generic(String),
}

impl From<crate::tools::calculator::CalcError> for ServiceError {
    fn from(err: crate::tools::calculator::CalcError) -> Self {
        match err {
            crate::tools::calculator::CalcError::InvalidCharacter(c) => ServiceError::InvalidCharacter(c),
            crate::tools::calculator::CalcError::MismatchedParens => ServiceError::MismatchedParens,
            crate::tools::calculator::CalcError::InvalidExpression => ServiceError::InvalidExpression("无效表达式".to_string()),
            crate::tools::calculator::CalcError::DivisionByZero => ServiceError::DivisionByZero,
            crate::tools::calculator::CalcError::UnexpectedEndOfExpression => ServiceError::UnexpectedEndOfExpression,
        }
    }
}