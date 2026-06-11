//! Operators opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute an operators opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::OperatorAdd => {
            let (a, b) = get_two_numbers(args)?;
            Ok(Value::Number(a + b))
        }
        Opcode::OperatorSubtract => {
            let (a, b) = get_two_numbers(args)?;
            Ok(Value::Number(a - b))
        }
        Opcode::OperatorMultiply => {
            let (a, b) = get_two_numbers(args)?;
            Ok(Value::Number(a * b))
        }
        Opcode::OperatorDivide => {
            let (a, b) = get_two_numbers(args)?;
            if b == 0.0 {
                Ok(Value::Number(f64::INFINITY))
            } else {
                Ok(Value::Number(a / b))
            }
        }
        Opcode::OperatorRandom => {
            let (from, to) = get_two_numbers(args)?;
            let min = from.min(to);
            let max = from.max(to);
            // Simple deterministic random for testing
            // In production, would use proper RNG
            let range = max - min;
            let elapsed = runtime.timer_elapsed();
            let pseudo = ((elapsed * 1000000.0).fract() * range + min).clamp(min, max);
            Ok(Value::Number(pseudo))
        }
        Opcode::OperatorGt => {
            let (a, b) = get_two_numbers(args)?;
            Ok(Value::Bool(a > b))
        }
        Opcode::OperatorLt => {
            let (a, b) = get_two_numbers(args)?;
            Ok(Value::Bool(a < b))
        }
        Opcode::OperatorEquals => {
            // Scratch-style equality: compare as strings if possible
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let a_str = items[0].as_string();
                    let b_str = items[1].as_string();
                    match (a_str, b_str) {
                        (Some(a), Some(b)) => Ok(Value::Bool(
                            a.to_lowercase() == b.to_lowercase(),
                        )),
                        _ => {
                            let a_num = items[0].as_number().unwrap_or(0.0);
                            let b_num = items[1].as_number().unwrap_or(0.0);
                            Ok(Value::Bool(a_num == b_num))
                        }
                    }
                }
                _ => Ok(Value::Bool(false)),
            }
        }
        Opcode::OperatorAnd => {
            let (a, b) = get_two_bools(args)?;
            Ok(Value::Bool(a && b))
        }
        Opcode::OperatorOr => {
            let (a, b) = get_two_bools(args)?;
            Ok(Value::Bool(a || b))
        }
        Opcode::OperatorNot => {
            let val = args.as_bool();
            Ok(Value::Bool(!val))
        }
        Opcode::OperatorJoin => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let a = items[0].as_string().unwrap_or_default();
                    let b = items[1].as_string().unwrap_or_default();
                    Ok(Value::String(format!("{}{}", a, b)))
                }
                _ => Ok(Value::String(String::new())),
            }
        }
        Opcode::OperatorLetterOf => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let idx = items[0].as_number().unwrap_or(1.0) as usize;
                    let s = items[1].as_string().unwrap_or_default();
                    if idx == 0 || idx > s.len() {
                        Ok(Value::String(String::new()))
                    } else {
                        Ok(Value::String(s.chars().nth(idx - 1).unwrap_or('\0').to_string()))
                    }
                }
                _ => Ok(Value::String(String::new())),
            }
        }
        Opcode::OperatorLength => {
            let s = args.as_string().unwrap_or_default();
            Ok(Value::Number(s.len() as f64))
        }
        Opcode::OperatorContains => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let string = items[0].as_string().unwrap_or_default();
                    let contains = items[1].as_string().unwrap_or_default();
                    Ok(Value::Bool(
                        string.to_lowercase().contains(&contains.to_lowercase()),
                    ))
                }
                _ => Ok(Value::Bool(false)),
            }
        }
        Opcode::OperatorMod => {
            let (a, b) = get_two_numbers(args)?;
            if b == 0.0 {
                Ok(Value::Number(0.0))
            } else {
                Ok(Value::Number(a % b))
            }
        }
        Opcode::OperatorRound => {
            let n = args.as_number().unwrap_or(0.0);
            Ok(Value::Number(n.round()))
        }
        Opcode::OperatorMathop => {
            let func = args.as_string().unwrap_or_default();
            // For testing, we'll just return the function name
            // In production, the actual math operation would be performed
            Ok(Value::String(func))
        }
        _ => Err(OpcodeError::UnknownOpcode(format!("{:?}", opcode))),
    }
}

/// Extract two numbers from a Value (typically a List with two items).
fn get_two_numbers(args: &Value) -> Result<(f64, f64), OpcodeError> {
    match args {
        Value::List(items) if items.len() >= 2 => {
            let a = items[0].as_number().unwrap_or(0.0);
            let b = items[1].as_number().unwrap_or(0.0);
            Ok((a, b))
        }
        Value::Number(n) => Ok((*n, 0.0)),
        _ => Ok((0.0, 0.0)),
    }
}

/// Extract two bools from a Value.
fn get_two_bools(args: &Value) -> Result<(bool, bool), OpcodeError> {
    match args {
        Value::List(items) if items.len() >= 2 => {
            let a = items[0].as_bool();
            let b = items[1].as_bool();
            Ok((a, b))
        }
        Value::Bool(b) => Ok((*b, false)),
        _ => Ok((false, false)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TargetState;

    fn make_runtime() -> RuntimeState {
        let mut runtime = RuntimeState::new();
        runtime.current_target = "Cat".to_string();
        runtime.add_target(TargetState::new_sprite("Cat"));
        runtime.start();
        runtime
    }

    #[test]
    fn test_operator_add() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorAdd,
            &mut runtime,
            &Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_operator_subtract() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorSubtract,
            &mut runtime,
            &Value::List(vec![Value::Number(10.0), Value::Number(3.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_operator_multiply() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorMultiply,
            &mut runtime,
            &Value::List(vec![Value::Number(6.0), Value::Number(7.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_operator_divide() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorDivide,
            &mut runtime,
            &Value::List(vec![Value::Number(20.0), Value::Number(4.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_operator_divide_by_zero() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorDivide,
            &mut runtime,
            &Value::List(vec![Value::Number(10.0), Value::Number(0.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(f64::INFINITY));
    }

    #[test]
    fn test_operator_gt() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorGt,
            &mut runtime,
            &Value::List(vec![Value::Number(5.0), Value::Number(3.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true));

        let result2 = execute(
            &Opcode::OperatorGt,
            &mut runtime,
            &Value::List(vec![Value::Number(3.0), Value::Number(5.0)]),
        )
        .expect("should execute");
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_operator_lt() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorLt,
            &mut runtime,
            &Value::List(vec![Value::Number(3.0), Value::Number(5.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_operator_equals() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorEquals,
            &mut runtime,
            &Value::List(vec![Value::Number(5.0), Value::Number(5.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_operator_not() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::OperatorNot, &mut runtime, &Value::Bool(true))
            .expect("should execute");
        assert_eq!(result, Value::Bool(false));

        let result2 = execute(&Opcode::OperatorNot, &mut runtime, &Value::Bool(false))
            .expect("should execute");
        assert_eq!(result2, Value::Bool(true));
    }

    #[test]
    fn test_operator_and() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorAnd,
            &mut runtime,
            &Value::List(vec![Value::Bool(true), Value::Bool(true)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true));

        let result2 = execute(
            &Opcode::OperatorAnd,
            &mut runtime,
            &Value::List(vec![Value::Bool(true), Value::Bool(false)]),
        )
        .expect("should execute");
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_operator_or() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorOr,
            &mut runtime,
            &Value::List(vec![Value::Bool(false), Value::Bool(true)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true));

        let result2 = execute(
            &Opcode::OperatorOr,
            &mut runtime,
            &Value::List(vec![Value::Bool(false), Value::Bool(false)]),
        )
        .expect("should execute");
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_operator_join() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorJoin,
            &mut runtime,
            &Value::List(vec![
                Value::String("hello".to_string()),
                Value::String(" world".to_string()),
            ]),
        )
        .expect("should execute");
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_operator_letter_of() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorLetterOf,
            &mut runtime,
            &Value::List(vec![
                Value::Number(1.0),
                Value::String("abc".to_string()),
            ]),
        )
        .expect("should execute");
        assert_eq!(result, Value::String("a".to_string()));

        let result2 = execute(
            &Opcode::OperatorLetterOf,
            &mut runtime,
            &Value::List(vec![
                Value::Number(3.0),
                Value::String("abc".to_string()),
            ]),
        )
        .expect("should execute");
        assert_eq!(result2, Value::String("c".to_string()));
    }

    #[test]
    fn test_operator_length() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorLength,
            &mut runtime,
            &Value::String("hello".to_string()),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_operator_contains() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorContains,
            &mut runtime,
            &Value::List(vec![
                Value::String("Hello World".to_string()),
                Value::String("world".to_string()),
            ]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true)); // Case-insensitive
    }

    #[test]
    fn test_operator_mod() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::OperatorMod,
            &mut runtime,
            &Value::List(vec![Value::Number(10.0), Value::Number(3.0)]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_operator_round() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::OperatorRound, &mut runtime, &Value::Number(3.7))
            .expect("should execute");
        assert_eq!(result, Value::Number(4.0));

        let result2 = execute(&Opcode::OperatorRound, &mut runtime, &Value::Number(3.2))
            .expect("should execute");
        assert_eq!(result2, Value::Number(3.0));
    }
}
