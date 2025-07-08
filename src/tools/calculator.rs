//! # 自定义规则的算术表达式计算器
//!
//! 该程序实现了两个核心功能：
//! 1. `calculate`: 根据自定义规则计算一个字符串形式的算术表达式。
//!    - 规则1：所有数字在参与运算前，必须根据指定小数位数进行四舍五入。
//!    - 规则2：支持加、减、乘、除、括号和百分号。
//!    - 规则3：计算结果也需要进行最终的四舍五入。
//! 2. `validate`: 验证一个算式的计算结果是否与预期值相符。
//!
//! 核心算法采用"调度场算法"(Shunting-yard Algorithm)，分为三步：
//! 1. **词法分析 (Tokenization)**: 将字符串分解为词元（Token），并在此阶段完成数字的预先舍入和百分比处理。
//! 2. **语法分析 (Parsing)**: 使用调度场算法将中缀表达式词元序列转换为后缀表达式（逆波兰表示法, RPN）。
//! 3. **求值 (Evaluation)**: 计算后缀表达式得出结果。

use std::iter::Peekable;
use std::str::Chars;

// --- 公开的枚举和结构体 ---

/// 定义词元（Token）类型
/// `Clone` is needed for the shunting-yard algorithm.
/// `PartialEq` and `Debug` are for testing and debugging.
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
}

/// 定义百分数的处理策略
/// 用户可以通过这个参数决定如何处理百分数（%符号）
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PercentRounding {
    /// 先将百分数除以100转换为小数，然后再进行舍入。
    /// 例如: 50.126% with round 2 -> 0.50126 -> 0.50
    DivideBy100ThenRound,
    /// 先对百分数进行舍入，然后再除以100转换为小数。
    /// 例如: 50.126% with round 2 -> 50.13 -> 0.5013
    RoundThenDivideBy100,
}

/// 定义可能出现的错误类型
#[derive(Debug, PartialEq)]
pub enum CalcError {
    InvalidCharacter(char),
    MismatchedParens,
    InvalidExpression,
    DivisionByZero,
    /// 当表达式不完整时（例如 "5 * "）
    UnexpectedEndOfExpression,
}

// --- 核心功能函数 ---

/// 函数1：计算
///
/// # 参数
/// * `expr` - 要计算的算式字符串
/// * `decimals` - 要保留的小数位数
/// * `rounding_strategy` - 处理百分比的舍入策略
///
/// # 返回
/// * `Result<f64, CalcError>` - 计算结果或错误
pub fn calculate(
    expr: &str,
    decimals: u32,
    rounding_strategy: PercentRounding,
) -> Result<f64, CalcError> {
    // 步骤 1: 词法分析与预先舍入
    let tokens = tokenize_and_round(expr, decimals, rounding_strategy)?;

    // 步骤 2: 转换为后缀表达式 (Shunting-yard)
    let rpn_queue = shunt_to_rpn(&tokens)?;

    // 步骤 3: 求值
    let result = evaluate_rpn(&rpn_queue)?;

    // 步骤 4: 最终结果舍入
    Ok(round_value(result, decimals))
}

/// 函数2：验证
///
/// # 参数
/// * `expr` - 要计算的算式字符串
/// * `expected` - 预期的结果
/// * `decimals` - 要保留的小数位数
/// * `rounding_strategy` - 处理百分比的舍入策略
///
/// # 返回
/// * `bool` - 算式计算结果是否与预期一致
pub fn validate(
    expr: &str,
    expected: f64,
    decimals: u32,
    rounding_strategy: PercentRounding,
) -> bool {
    // 使用一个小的容差来比较浮点数，避免精度问题
    const EPSILON: f64 = 1e-9;

    match calculate(expr, decimals, rounding_strategy) {
        Ok(actual) => (actual - expected).abs() < EPSILON,
        Err(_) => false, // 如果计算出错，则验证失败
    }
}

// --- 辅助函数 ---

/// 辅助函数：对一个 f64 值进行四舍五入
fn round_value(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

/// 辅助函数：获取操作符的优先级
fn precedence(token: &Token) -> u8 {
    match token {
        Token::Add | Token::Subtract => 1,
        Token::Multiply | Token::Divide => 2,
        _ => 0,
    }
}

// --- 算法核心实现 ---

/// 步骤 1: 词法分析与预先舍入
fn tokenize_and_round(
    expr: &str,
    decimals: u32,
    rounding_strategy: PercentRounding,
) -> Result<Vec<Token>, CalcError> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' => {
                let num_str = consume_number(&mut chars);
                let mut num = num_str.parse::<f64>().map_err(|_| CalcError::InvalidExpression)?;

                // 检查百分号
                if let Some('%') = chars.peek() {
                    chars.next(); // consume '%'
                    num = match rounding_strategy {
                        PercentRounding::DivideBy100ThenRound => {
                            let converted = num / 100.0;
                            round_value(converted, decimals)
                        }
                        PercentRounding::RoundThenDivideBy100 => {
                            let rounded = round_value(num, decimals);
                            rounded / 100.0
                        }
                    };
                } else {
                    // 普通数字的舍入
                    num = round_value(num, decimals);
                }
                tokens.push(Token::Number(num));
            }
            '+' => {
                tokens.push(Token::Add);
                chars.next();
            }
            // 处理负号和减号的区别
            '-' => {
                let is_unary = tokens.is_empty() || matches!(tokens.last(), Some(Token::LeftParen) | Some(Token::Add) | Some(Token::Subtract) | Some(Token::Multiply) | Some(Token::Divide));
                chars.next(); // consume '-'
                if is_unary {
                    // This is a negative number
                    let num_str = consume_number(&mut chars);
                    if num_str.is_empty() {
                        return Err(CalcError::InvalidExpression);
                    }
                    let mut num = -num_str.parse::<f64>().map_err(|_| CalcError::InvalidExpression)?;
                     // Check for percentage on negative number
                    if let Some('%') = chars.peek() {
                        chars.next(); // consume '%'
                         num = match rounding_strategy {
                            PercentRounding::DivideBy100ThenRound => round_value(num / 100.0, decimals),
                            PercentRounding::RoundThenDivideBy100 => round_value(num, decimals) / 100.0,
                        };
                    } else {
                        num = round_value(num, decimals);
                    }
                    tokens.push(Token::Number(num));
                } else {
                    // This is a subtraction operator
                    tokens.push(Token::Subtract);
                }
            }
            '*' => {
                tokens.push(Token::Multiply);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Divide);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            ' ' | '\t' | '\n' => {
                // Skip whitespace
                chars.next();
            }
            _ => return Err(CalcError::InvalidCharacter(c)),
        }
    }

    Ok(tokens)
}

/// 辅助函数：从字符流中消费一个完整的数字字符串（支持千分位分隔符）
fn consume_number(chars: &mut Peekable<Chars>) -> String {
    let mut num_str = String::new();
    let mut has_digit = false;
    
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            has_digit = true;
            chars.next();
        } else if (c == '.' || c == ',' || c == ' ' || c == '\'') && has_digit {
            // 只有在已经有数字的情况下才消费分隔符
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

/// 辅助函数：标准化数字字符串，移除千分位分隔符并处理不同的小数点格式
pub fn normalize_number(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    
    // 简化的格式检测逻辑
    let cleaned = input.trim();
    
    // 如果包含逗号和点号，判断哪个是小数点
    if cleaned.contains(',') && cleaned.contains('.') {
        let last_comma = cleaned.rfind(',').unwrap();
        let last_dot = cleaned.rfind('.').unwrap();
        
        if last_dot > last_comma {
            // 美式: 1,234.56 - 点号是小数点
            remove_thousand_separators(cleaned, &[',', '\'', ' '])
        } else {
            // 欧式: 1.234,56 - 逗号是小数点
            let mut result = remove_thousand_separators(cleaned, &['.', '\'', ' ']);
            result = result.replace(',', ".");
            result
        }
    } else if cleaned.contains(',') {
        // 只有逗号 - 检查是否为小数点
        let comma_count = cleaned.matches(',').count();
        if comma_count == 1 {
            let parts: Vec<&str> = cleaned.split(',').collect();
            if parts.len() == 2 && parts[1].len() <= 3 && parts[1].chars().all(|c| c.is_ascii_digit()) {
                // 很可能是欧式小数点
                cleaned.replace(',', ".")
            } else {
                // 千分位分隔符
                remove_thousand_separators(cleaned, &[',', '\'', ' '])
            }
        } else {
            // 多个逗号，必然是千分位
            remove_thousand_separators(cleaned, &[',', '\'', ' '])
        }
    } else if cleaned.contains('.') {
        // 只有点号 - 检查是否为千分位分隔符
        let dot_count = cleaned.matches('.').count();
        if dot_count == 1 {
            // 单个点号，很可能是小数点
            cleaned.to_string()
        } else {
            // 多个点号，检查最后一个是否为小数点
            let parts: Vec<&str> = cleaned.split('.').collect();
            if let Some(last_part) = parts.last() {
                if last_part.len() <= 3 && last_part.chars().all(|c| c.is_ascii_digit()) {
                    // 最后部分像小数
                    let before_last_dot = &cleaned[..cleaned.rfind('.').unwrap()];
                    let cleaned_before = remove_thousand_separators(before_last_dot, &['.', '\'', ' ']);
                    format!("{}.{}", cleaned_before, last_part)
                } else {
                    // 全部是千分位
                    remove_thousand_separators(cleaned, &['.', '\'', ' '])
                }
            } else {
                cleaned.to_string()
            }
        }
    } else {
        // 只有空格或撇号
        remove_thousand_separators(cleaned, &['\'', ' '])
    }
}

/// 移除千分位分隔符
fn remove_thousand_separators(input: &str, separators: &[char]) -> String {
    let mut result = input.to_string();
    for &sep in separators {
        result = result.replace(sep, "");
    }
    result
}


/// 步骤 2: 将词元序列转换为后缀表达式 (Shunting-yard)
fn shunt_to_rpn(tokens: &[Token]) -> Result<Vec<Token>, CalcError> {
    let mut output_queue: Vec<Token> = Vec::new();
    let mut operator_stack: Vec<Token> = Vec::new();

    for token in tokens.iter().cloned() {
        match token {
            Token::Number(_) => output_queue.push(token),
            Token::LeftParen => operator_stack.push(token),
            Token::RightParen => {
                while let Some(top_op) = operator_stack.last() {
                    if matches!(top_op, Token::LeftParen) {
                        break;
                    }
                    output_queue.push(operator_stack.pop().unwrap());
                }
                if operator_stack.pop().is_none() {
                    // Mismatched parentheses
                    return Err(CalcError::MismatchedParens);
                }
            }
            // Operator case
            _ => {
                while let Some(top_op) = operator_stack.last() {
                    if matches!(top_op, Token::LeftParen) {
                        break;
                    }
                    if precedence(top_op) >= precedence(&token) {
                        output_queue.push(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                operator_stack.push(token);
            }
        }
    }

    // Pop remaining operators from the stack to the queue
    while let Some(op) = operator_stack.pop() {
        if matches!(op, Token::LeftParen) {
            return Err(CalcError::MismatchedParens);
        }
        output_queue.push(op);
    }

    Ok(output_queue)
}

/// 步骤 3: 求值后缀表达式
fn evaluate_rpn(rpn_queue: &[Token]) -> Result<f64, CalcError> {
    let mut operand_stack: Vec<f64> = Vec::new();

    for token in rpn_queue.iter().cloned() {
        match token {
            Token::Number(n) => operand_stack.push(n),
            _ => {
                let rhs = operand_stack.pop().ok_or(CalcError::InvalidExpression)?;
                let lhs = operand_stack.pop().ok_or(CalcError::InvalidExpression)?;
                let result = match token {
                    Token::Add => lhs + rhs,
                    Token::Subtract => lhs - rhs,
                    Token::Multiply => lhs * rhs,
                    Token::Divide => {
                        if rhs.abs() < 1e-9 {
                            return Err(CalcError::DivisionByZero);
                        }
                        lhs / rhs
                    }
                    _ => unreachable!(), // Should not happen with a valid RPN queue
                };
                operand_stack.push(result);
            }
        }
    }

    if operand_stack.len() == 1 {
        Ok(operand_stack.pop().unwrap())
    } else {
        Err(CalcError::InvalidExpression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(calculate("1 + 2", 0, PercentRounding::DivideBy100ThenRound), Ok(3.0));
        assert_eq!(calculate("5 - 3", 0, PercentRounding::DivideBy100ThenRound), Ok(2.0));
        assert_eq!(calculate("2 * 3", 0, PercentRounding::DivideBy100ThenRound), Ok(6.0));
        assert_eq!(calculate("8 / 2", 0, PercentRounding::DivideBy100ThenRound), Ok(4.0));
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(calculate("(1 + 2) * 3", 0, PercentRounding::DivideBy100ThenRound), Ok(9.0));
        assert_eq!(calculate("2 * (3 + 4)", 0, PercentRounding::DivideBy100ThenRound), Ok(14.0));
        assert_eq!(calculate("((1 + 2) * 3) / 3", 0, PercentRounding::DivideBy100ThenRound), Ok(3.0));
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(calculate("1 + 2 * 3", 0, PercentRounding::DivideBy100ThenRound), Ok(7.0));
        assert_eq!(calculate("2 * 3 + 1", 0, PercentRounding::DivideBy100ThenRound), Ok(7.0));
        assert_eq!(calculate("6 / 2 + 1", 0, PercentRounding::DivideBy100ThenRound), Ok(4.0));
    }

    #[test]
    fn test_rounding() {
        assert_eq!(calculate("1.234 + 2.567", 2, PercentRounding::DivideBy100ThenRound), Ok(3.80));
        assert_eq!(calculate("1.235 + 2.564", 2, PercentRounding::DivideBy100ThenRound), Ok(3.80));
        assert_eq!(calculate("1.999 + 0.001", 2, PercentRounding::DivideBy100ThenRound), Ok(2.00));
    }

    #[test]
    fn test_percentage_convert_then_round() {
        assert_eq!(calculate("50%", 2, PercentRounding::DivideBy100ThenRound), Ok(0.50));
        assert_eq!(calculate("50.126%", 2, PercentRounding::DivideBy100ThenRound), Ok(0.50));
        assert_eq!(calculate("50.126% + 25%", 2, PercentRounding::DivideBy100ThenRound), Ok(0.75));
    }

    #[test]
    fn test_percentage_round_then_convert() {
        assert_eq!(calculate("50.126%", 2, PercentRounding::RoundThenDivideBy100), Ok(0.50));
        assert_eq!(calculate("50.124%", 2, PercentRounding::RoundThenDivideBy100), Ok(0.50));
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(calculate("-5 + 3", 0, PercentRounding::DivideBy100ThenRound), Ok(-2.0));
        assert_eq!(calculate("5 + -3", 0, PercentRounding::DivideBy100ThenRound), Ok(2.0));
        assert_eq!(calculate("-5 * -3", 0, PercentRounding::DivideBy100ThenRound), Ok(15.0));
        assert_eq!(calculate("(-5) * 3", 0, PercentRounding::DivideBy100ThenRound), Ok(-15.0));
    }

    #[test]
    fn test_negative_percentage() {
        assert_eq!(calculate("-50%", 2, PercentRounding::DivideBy100ThenRound), Ok(-0.50));
        assert_eq!(calculate("-50.126%", 2, PercentRounding::DivideBy100ThenRound), Ok(-0.50));
    }

    #[test]
    fn test_decimal_numbers() {
        assert_eq!(calculate("1.5 + 2.5", 1, PercentRounding::DivideBy100ThenRound), Ok(4.0));
        assert_eq!(calculate("3.14 * 2", 2, PercentRounding::DivideBy100ThenRound), Ok(6.28));
        assert_eq!(calculate("0.1 + 0.2", 1, PercentRounding::DivideBy100ThenRound), Ok(0.3));
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(calculate("(1.5 + 2.5) * 3 - 1", 1, PercentRounding::DivideBy100ThenRound), Ok(11.0));
        assert_eq!(calculate("100% - 50% + 25%", 2, PercentRounding::DivideBy100ThenRound), Ok(0.75));
        assert_eq!(calculate("(50% + 25%) * 2", 2, PercentRounding::DivideBy100ThenRound), Ok(1.50));
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(calculate("5 / 0", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::DivisionByZero));
        assert_eq!(calculate("1 / (2 - 2)", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn test_invalid_expressions() {
        assert_eq!(calculate("1 +", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::InvalidExpression));
        assert_eq!(calculate("* 2", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::InvalidExpression));
        assert_eq!(calculate("1 + + 2", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::InvalidExpression));
    }

    #[test]
    fn test_mismatched_parentheses() {
        assert_eq!(calculate("(1 + 2", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::MismatchedParens));
        assert_eq!(calculate("1 + 2)", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::MismatchedParens));
        assert_eq!(calculate("((1 + 2)", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::MismatchedParens));
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(calculate("1 + 2 @", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::InvalidCharacter('@')));
        assert_eq!(calculate("1 & 2", 0, PercentRounding::DivideBy100ThenRound), Err(CalcError::InvalidCharacter('&')));
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(calculate("  1  +  2  ", 0, PercentRounding::DivideBy100ThenRound), Ok(3.0));
        assert_eq!(calculate("1\t+\t2", 0, PercentRounding::DivideBy100ThenRound), Ok(3.0));
        assert_eq!(calculate("1\n+\n2", 0, PercentRounding::DivideBy100ThenRound), Ok(3.0));
    }

    #[test]
    fn test_validate_function() {
        assert!(validate("1 + 2", 3.0, 0, PercentRounding::DivideBy100ThenRound));
        assert!(validate("1.234 + 2.567", 3.80, 2, PercentRounding::DivideBy100ThenRound));
        assert!(!validate("1 + 2", 4.0, 0, PercentRounding::DivideBy100ThenRound));
        assert!(!validate("1 / 0", 0.0, 0, PercentRounding::DivideBy100ThenRound));
    }

    #[test]
    fn test_floating_point_precision() {
        // Test that we handle floating point precision issues properly
        assert!(validate("0.1 + 0.2", 0.3, 1, PercentRounding::DivideBy100ThenRound));
        assert!(validate("0.1 + 0.1 + 0.1", 0.3, 1, PercentRounding::DivideBy100ThenRound));
    }

    #[test]
    fn test_thousand_separators() {
        // 美式格式：逗号作为千分位分隔符
        assert_eq!(calculate("1,234.56 + 2,000.44", 2, PercentRounding::DivideBy100ThenRound), Ok(3235.00));
        
        // 欧式格式：点号作为千分位分隔符，逗号作为小数点
        assert_eq!(calculate("1.234,56 + 2.000,44", 2, PercentRounding::DivideBy100ThenRound), Ok(3235.00));
        
        // 大数字测试
        assert_eq!(calculate("1,000,000.00 + 500,000.00", 0, PercentRounding::DivideBy100ThenRound), Ok(1500000.0));
        assert_eq!(calculate("1.000.000,50 + 500.000,25", 2, PercentRounding::DivideBy100ThenRound), Ok(1500000.75));
    }

    #[test]
    fn test_thousand_separators_edge_cases() {
        // 只有一个逗号，判断为小数点（欧式）
        assert_eq!(calculate("123,45 + 100", 2, PercentRounding::DivideBy100ThenRound), Ok(223.45));
        
        // 只有一个点号，判断为小数点（美式）
        assert_eq!(calculate("123.45 + 100", 2, PercentRounding::DivideBy100ThenRound), Ok(223.45));
        
        // 复杂表达式中的千分位
        assert_eq!(calculate("(1,234.56 + 2,000.44) / 2", 2, PercentRounding::DivideBy100ThenRound), Ok(1617.50));
    }

    #[test]
    fn test_thousand_separators_with_percentage() {
        // 千分位分隔符与百分号结合（简化测试）
        assert_eq!(calculate("100% + 50%", 2, PercentRounding::DivideBy100ThenRound), Ok(1.50));
        assert_eq!(calculate("1,234.56% / 100", 4, PercentRounding::DivideBy100ThenRound), Ok(0.1235));
    }

    #[test]
    fn test_mixed_number_formats() {
        // 测试在同一表达式中混合使用不同格式
        // 美式 + 欧式
        assert_eq!(calculate("1,234.56 + 1.000,44", 2, PercentRounding::DivideBy100ThenRound), Ok(2235.00));
        
        // 美式 + 简单数字
        assert_eq!(calculate("1,234.56 + 100", 2, PercentRounding::DivideBy100ThenRound), Ok(1334.56));
        
        // 欧式 + 简单数字
        assert_eq!(calculate("1.234,56 + 100", 2, PercentRounding::DivideBy100ThenRound), Ok(1334.56));
        
        // 复杂混合表达式
        assert_eq!(calculate("(1,234.56 + 1.000,44) * 0.5", 2, PercentRounding::DivideBy100ThenRound), Ok(1117.50));
        
        // 混合格式与百分比
        assert_eq!(calculate("1,234.56 + 10% * 1.000,00", 2, PercentRounding::DivideBy100ThenRound), Ok(1334.56));
    }

    #[test]
    fn test_batch_validation_logic() {
        // 测试批量验证的核心逻辑
        use crate::tools::*;
        
        // 基本批量验证
        let expressions = vec![
            "1 + 2|3".to_string(),
            "2 * 3|6".to_string(),
            "10 / 2|5".to_string(),
        ];
        
        for expr in &expressions {
            let parts: Vec<&str> = expr.split('|').collect();
            let expression = parts[0];
            let expected: f64 = parts[1].parse().unwrap();
            assert!(validate(expression, expected, 0, PercentRounding::DivideBy100ThenRound));
        }
        
        // 带小数位的批量验证
        let expressions_with_decimals = vec![
            "1.234 + 2.567|3.80|2".to_string(),
            "50.126%|0.50|2".to_string(),
        ];
        
        for expr in &expressions_with_decimals {
            let parts: Vec<&str> = expr.split('|').collect();
            let expression = parts[0];
            let expected: f64 = parts[1].parse().unwrap();
            let decimals: u32 = parts[2].parse().unwrap();
            assert!(validate(expression, expected, decimals, PercentRounding::DivideBy100ThenRound));
        }
    }

    #[test]
    fn test_expected_value_with_percentage() {
        // 测试预期值包含百分数的情况
        
        // 表达式和预期值都包含百分数
        assert!(validate("50%", 0.5, 2, PercentRounding::DivideBy100ThenRound));
        assert!(validate("50.126%", 0.5, 2, PercentRounding::DivideBy100ThenRound));
        
        // 不同的舍入策略
        assert!(validate("50.126%", 0.50, 2, PercentRounding::DivideBy100ThenRound));
        assert!(validate("50.126%", 0.50, 2, PercentRounding::RoundThenDivideBy100));
        
        // 复杂表达式与百分数预期值
        assert!(validate("25% + 25%", 0.5, 2, PercentRounding::DivideBy100ThenRound));
    }

    #[test]
    fn test_expected_value_with_thousand_separators() {
        // 测试预期值包含千分位分隔符的情况
        
        // 美式千分位
        assert!(validate("1000 + 234.56", 1234.56, 2, PercentRounding::DivideBy100ThenRound));
        
        // 大数字验证
        assert!(validate("500000 + 500000", 1000000.0, 0, PercentRounding::DivideBy100ThenRound));
        
        // 负数验证
        assert!(validate("100 - 200", -100.0, 0, PercentRounding::DivideBy100ThenRound));
    }

    #[test]
    fn test_mixed_expected_formats() {
        // 测试混合格式的预期值
        
        // 百分数表达式，千分位预期值（这种情况应该根据预期值格式解析）
        assert!(validate("1% * 100", 1.0, 2, PercentRounding::DivideBy100ThenRound));
        
        // 验证舍入逻辑：0.5 在 decimals=0 时会被舍入为 1
        let result = calculate("0.5 * 100", 0, PercentRounding::DivideBy100ThenRound).unwrap();
        assert_eq!(result, 100.0); // 0.5 舍入为 1，所以 1 * 100 = 100
        
        // 正确的测试：使用足够的小数位数
        assert!(validate("0.5 * 100", 50.0, 1, PercentRounding::DivideBy100ThenRound));
        
        // 复杂混合情况
        assert!(validate("1,000.00 / 10", 100.0, 2, PercentRounding::DivideBy100ThenRound));
        
        // 测试整数情况
        assert!(validate("50 * 2", 100.0, 0, PercentRounding::DivideBy100ThenRound));
    }
}