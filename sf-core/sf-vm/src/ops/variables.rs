//! Variables opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute a variables opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::DataSetVariableTo => {
            // Args is a List: [variable_name, value]
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let name = items[0].as_string().unwrap_or_default();
                    let value = items[1].clone();
                    runtime.set_variable_scoped(&name, value);
                    Ok(Value::Null)
                }
                _ => Err(OpcodeError::InvalidArgument {
                    opcode: "data_setvariableto".to_string(),
                    message: "expected list with [name, value]".to_string(),
                }),
            }
        }
        Opcode::DataChangeVariableBy => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let name = items[0].as_string().unwrap_or_default();
                    let delta = items[1].as_number().unwrap_or(0.0);
                    let current = runtime
                        .get_variable_scoped(&name)
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    runtime.set_variable_scoped(&name, Value::Number(current + delta));
                    Ok(Value::Null)
                }
                _ => Err(OpcodeError::InvalidArgument {
                    opcode: "data_changevariableby".to_string(),
                    message: "expected list with [name, delta]".to_string(),
                }),
            }
        }
        Opcode::DataVariable => {
            let name = args.as_string().unwrap_or_default();
            Ok(runtime
                .get_variable_scoped(&name)
                .cloned()
                .unwrap_or(Value::Number(0.0)))
        }
        Opcode::DataShowVariable | Opcode::DataHideVariable => {
            // UI operations - acknowledged but not implemented in headless VM
            Ok(Value::Null)
        }
        Opcode::DataAddToList => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let list_name = items[0].as_string().unwrap_or_default();
                    let item = items[1].clone();
                    // Lists are stored as variables with List values
                    let current = runtime.get_variable_scoped(&list_name).cloned();
                    match current {
                        Some(Value::List(mut list)) => {
                            list.push(item);
                            runtime.set_variable_scoped(&list_name, Value::List(list));
                        }
                        _ => {
                            runtime.set_variable_scoped(&list_name, Value::List(vec![item]));
                        }
                    }
                    Ok(Value::Null)
                }
                _ => Err(OpcodeError::InvalidArgument {
                    opcode: "data_addtolist".to_string(),
                    message: "expected list with [list_name, item]".to_string(),
                }),
            }
        }
        Opcode::DataDeleteOfList => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let list_name = items[0].as_string().unwrap_or_default();
                    let index = items[1].as_number().unwrap_or(1.0) as usize;
                    let current = runtime.get_variable_scoped(&list_name).cloned();
                    if let Some(Value::List(mut list)) = current {
                        // 1-indexed, "all" means delete all
                        if index > 0 && index <= list.len() {
                            list.remove(index - 1);
                            runtime.set_variable_scoped(&list_name, Value::List(list));
                        }
                    }
                    Ok(Value::Null)
                }
                _ => Ok(Value::Null),
            }
        }
        Opcode::DataDeleteAllOfList => {
            let list_name = args.as_string().unwrap_or_default();
            runtime.set_variable_scoped(&list_name, Value::List(vec![]));
            Ok(Value::Null)
        }
        Opcode::DataInsertAtList => {
            match args {
                Value::List(items) if items.len() >= 3 => {
                    let list_name = items[0].as_string().unwrap_or_default();
                    let index = items[1].as_number().unwrap_or(1.0) as usize;
                    let item = items[2].clone();
                    let current = runtime.get_variable_scoped(&list_name).cloned();
                    if let Some(Value::List(mut list)) = current {
                        let insert_at = if index == 0 { 0 } else { (index - 1).min(list.len()) };
                        list.insert(insert_at, item);
                        runtime.set_variable_scoped(&list_name, Value::List(list));
                    }
                    Ok(Value::Null)
                }
                _ => Ok(Value::Null),
            }
        }
        Opcode::DataReplaceItemOfList => {
            match args {
                Value::List(items) if items.len() >= 3 => {
                    let list_name = items[0].as_string().unwrap_or_default();
                    let index = items[1].as_number().unwrap_or(1.0) as usize;
                    let item = items[2].clone();
                    let current = runtime.get_variable_scoped(&list_name).cloned();
                    if let Some(Value::List(mut list)) = current {
                        if index > 0 && index <= list.len() {
                            list[index - 1] = item;
                            runtime.set_variable_scoped(&list_name, Value::List(list));
                        }
                    }
                    Ok(Value::Null)
                }
                _ => Ok(Value::Null),
            }
        }
        Opcode::DataItemOfList => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let list_name = items[0].as_string().unwrap_or_default();
                    let index = items[1].as_number().unwrap_or(1.0) as usize;
                    let current = runtime.get_variable_scoped(&list_name);
                    if let Some(Value::List(list)) = current {
                        if index > 0 && index <= list.len() {
                            Ok(list[index - 1].clone())
                        } else {
                            Ok(Value::String(String::new()))
                        }
                    } else {
                        Ok(Value::String(String::new()))
                    }
                }
                _ => Ok(Value::String(String::new())),
            }
        }
        Opcode::DataLengthOfList => {
            let list_name = args.as_string().unwrap_or_default();
            let current = runtime.get_variable_scoped(&list_name);
            if let Some(Value::List(list)) = current {
                Ok(Value::Number(list.len() as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        }
        Opcode::DataListContainsItem => {
            match args {
                Value::List(items) if items.len() >= 2 => {
                    let list_name = items[0].as_string().unwrap_or_default();
                    let search_item = &items[1];
                    let current = runtime.get_variable_scoped(&list_name);
                    if let Some(Value::List(list)) = current {
                        Ok(Value::Bool(list.contains(search_item)))
                    } else {
                        Ok(Value::Bool(false))
                    }
                }
                _ => Ok(Value::Bool(false)),
            }
        }
        Opcode::DataShowList | Opcode::DataHideList => {
            Ok(Value::Null)
        }
        _ => Err(OpcodeError::UnknownOpcode(format!("{:?}", opcode))),
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
        runtime
    }

    #[test]
    fn test_data_set_variable() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::DataSetVariableTo,
            &mut runtime,
            &Value::List(vec![
                Value::String("score".to_string()),
                Value::Number(100.0),
            ]),
        )
        .expect("should execute");
        assert_eq!(
            runtime.get_variable_scoped("score"),
            Some(&Value::Number(100.0))
        );
    }

    #[test]
    fn test_data_change_variable() {
        let mut runtime = make_runtime();
        // Set initial value
        runtime.set_variable_scoped("score", Value::Number(50.0));
        execute(
            &Opcode::DataChangeVariableBy,
            &mut runtime,
            &Value::List(vec![
                Value::String("score".to_string()),
                Value::Number(25.0),
            ]),
        )
        .expect("should execute");
        assert_eq!(
            runtime.get_variable_scoped("score"),
            Some(&Value::Number(75.0))
        );
    }

    #[test]
    fn test_data_change_variable_from_zero() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::DataChangeVariableBy,
            &mut runtime,
            &Value::List(vec![
                Value::String("counter".to_string()),
                Value::Number(1.0),
            ]),
        )
        .expect("should execute");
        assert_eq!(
            runtime.get_variable_scoped("counter"),
            Some(&Value::Number(1.0))
        );
    }

    #[test]
    fn test_data_variable() {
        let mut runtime = make_runtime();
        runtime.set_variable_scoped("x", Value::Number(42.0));
        let result = execute(
            &Opcode::DataVariable,
            &mut runtime,
            &Value::String("x".to_string()),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_data_variable_not_found() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::DataVariable,
            &mut runtime,
            &Value::String("nonexistent".to_string()),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_data_add_to_list() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::DataAddToList,
            &mut runtime,
            &Value::List(vec![
                Value::String("mylist".to_string()),
                Value::String("item1".to_string()),
            ]),
        )
        .expect("should execute");
        execute(
            &Opcode::DataAddToList,
            &mut runtime,
            &Value::List(vec![
                Value::String("mylist".to_string()),
                Value::String("item2".to_string()),
            ]),
        )
        .expect("should execute");

        if let Some(Value::List(list)) = runtime.get_variable_scoped("mylist") {
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::String("item1".to_string()));
            assert_eq!(list[1], Value::String("item2".to_string()));
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn test_data_length_of_list() {
        let mut runtime = make_runtime();
        runtime.set_variable_scoped("mylist", Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]));
        let result = execute(
            &Opcode::DataLengthOfList,
            &mut runtime,
            &Value::String("mylist".to_string()),
        )
        .expect("should execute");
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_data_item_of_list() {
        let mut runtime = make_runtime();
        runtime.set_variable_scoped("mylist", Value::List(vec![
            Value::String("first".to_string()),
            Value::String("second".to_string()),
        ]));
        let result = execute(
            &Opcode::DataItemOfList,
            &mut runtime,
            &Value::List(vec![
                Value::String("mylist".to_string()),
                Value::Number(1.0),
            ]),
        )
        .expect("should execute");
        assert_eq!(result, Value::String("first".to_string()));
    }

    #[test]
    fn test_data_list_contains() {
        let mut runtime = make_runtime();
        runtime.set_variable_scoped("mylist", Value::List(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
        ]));
        let result = execute(
            &Opcode::DataListContainsItem,
            &mut runtime,
            &Value::List(vec![
                Value::String("mylist".to_string()),
                Value::String("apple".to_string()),
            ]),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_data_delete_of_list() {
        let mut runtime = make_runtime();
        runtime.set_variable_scoped("mylist", Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ]));
        execute(
            &Opcode::DataDeleteOfList,
            &mut runtime,
            &Value::List(vec![
                Value::String("mylist".to_string()),
                Value::Number(2.0),
            ]),
        )
        .expect("should execute");
        if let Some(Value::List(list)) = runtime.get_variable_scoped("mylist") {
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::String("a".to_string()));
            assert_eq!(list[1], Value::String("c".to_string()));
        }
    }
}
