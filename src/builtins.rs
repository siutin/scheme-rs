use std::collections::HashMap;
use std::rc::Rc;

use log::debug;

use crate::types::{Function, DataType, FloatIterExt};
use crate::env::{Env, EnvRef};
use crate::eval::{eval, datatype2str};
use crate::SchemeError;

#[macro_export]
macro_rules! define_comparison {
    ($proc:ident, $name:pat, $func:expr) => {
        let $proc = DataType::Proc(Function( Rc::new(|vec: Vec<DataType>, _: EnvRef| {
                debug!("Function - name: {:?} - Args: {:?}", stringify!($name), vec);
                if vec.len() != 2 {
                    return Err("function requires 2 arguments only".into());
                }
                let a = vec.get(0);
                let b = vec.get(1);

                if let (Some(a0), Some(b0)) = (a, b) {
                    if let (Some(a1), Some(b1)) = (a0.as_f64(), b0.as_f64()) {
                        let desc = format!("{} {} {}", a1, stringify!($name), b1);
                        debug!("Description: {}", desc);
                        Ok(Some(DataType::Bool($func(a1, b1))))
                    } else {
                        return Err("wrong argument datatype".into());
                    }
                } else {
                    return Err("wrong argument datatype".into());
                }

            })));
    };
}

/// Helper: car of a DataType (pair/list), returns None if not applicable.
fn car(d: &DataType) -> Option<DataType> {
    if let DataType::List(ref v) = d {
        if !v.is_empty() { return Some(v[0].clone()); }
    }
    None
}

/// Helper: cdr of a DataType (pair/list), returns None if not applicable.
fn cdr(d: &DataType) -> Option<DataType> {
    if let DataType::List(ref v) = d {
        if v.len() > 1 { return Some(DataType::List(v[1..].to_vec())); }
        if v.len() == 1 { return Some(DataType::List(Vec::new())); }
    }
    None
}

pub fn setup() -> HashMap<String, DataType> {
    let mut map = HashMap::new();
    map.insert("pi".to_string(), DataType::Float(std::f64::consts::PI));

    map.insert("+".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "+", vec);
        if !vec.iter().all(|x| x.is_number()) {
            return Err("wrong argument datatype".into());
        }
        // If all integers, stay integer; otherwise promote to float
        let all_int = vec.iter().all(|x| x.is_integer());
        if all_int {
            let sum: i64 = vec.iter().filter_map(|x| if let &DataType::Integer(i) = x { Some(i) } else { None }).sum();
            Ok(Some(DataType::Integer(sum)))
        } else {
            let sum: f64 = vec.iter().filter_map(|x| x.as_f64()).sum();
            Ok(Some(DataType::Float(sum)))
        }
    }))));

    map.insert("-".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "-", vec);
        if !vec.iter().all(|x| x.is_number()) {
            return Err("wrong argument datatype".into());
        }
        if vec.is_empty() {
            return Err("- function requires at least one argument".into());
        }
        let all_int = vec.iter().all(|x| x.is_integer());
        if all_int {
            let ints: Vec<i64> = vec.iter().filter_map(|x| if let &DataType::Integer(i) = x { Some(i) } else { None }).collect();
            let value = if ints.len() == 1 { -ints[0] } else { ints[1..].iter().fold(ints[0], |acc, x| acc - x) };
            Ok(Some(DataType::Integer(value)))
        } else {
            let nums: Vec<f64> = vec.iter().filter_map(|x| x.as_f64()).collect();
            let value = if nums.len() == 1 { -nums[0] } else { nums[1..].iter().fold(nums[0], |acc, x| acc - x) };
            Ok(Some(DataType::Float(value)))
        }
    }))));

    map.insert("*".to_string(), DataType::Proc(
        Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
            debug!("Function - name: {:?} - Args: {:?}", "*", vec);
            if !vec.iter().all(|x| x.is_number()) {
                return Err("wrong argument datatype".into());
            }
            let all_int = vec.iter().all(|x| x.is_integer());
            if all_int {
                let product: i64 = vec.iter().filter_map(|x| if let &DataType::Integer(i) = x { Some(i) } else { None }).product();
                Ok(Some(DataType::Integer(product)))
            } else {
                let product: f64 = vec.iter().filter_map(|x| x.as_f64()).product();
                Ok(Some(DataType::Float(product)))
            }
        }))));

    map.insert("/".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "/", vec);
        if !vec.iter().all(|x| x.is_number()) {
            return Err("wrong argument datatype".into());
        }
        if vec.is_empty() {
            return Err("/ function requires at least one argument".into());
        }

        let numbers: Vec<f64> = vec.iter().filter_map(|x| x.as_f64()).collect();

        // Check for division by zero in any divisor
        if numbers.len() > 1 && numbers[1..].iter().any(|&x| x == 0.0) {
            return Err(SchemeError::DivisionByZero);
        }

        // Division always returns Float (may produce non-integers)
        let value: f64 = if numbers.len() == 1 {
            1.0 / numbers[0]
        } else {
            numbers[1..].iter().fold(numbers[0], |acc, x| acc / x)
        };
        Ok(Some(DataType::Float(value)))
    }))));

    define_comparison!(gt, ">", |a,b| { a > b });
    map.insert(">".to_string(), gt);

    define_comparison!(lt, "<", |a,b| { a < b });
    map.insert("<".to_string(), lt);

    define_comparison!(eq, "=", |a,b| { a == b });
    map.insert("=".to_string(), eq);

    define_comparison!(ge, ">=", |a,b| { a >= b });
    map.insert(">=".to_string(), ge);

    define_comparison!(le, "<=", |a,b| { a <= b });
    map.insert("<=".to_string(), le);

    map.insert("abs".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "abs", vec);
        if vec.len() != 1 {
            return Err("abs function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("abs function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Integer(i) => Ok(Some(DataType::Integer(i.abs()))),
            &DataType::Float(f) => Ok(Some(DataType::Float(f.abs()))),
            _ => Err("abs function requires an argument of type 'number'".into())
        }
    }))));

    map.insert("append".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "append", vec);

        if vec.is_empty() {
            return Ok(Some(DataType::List(vec![])));
        }

        if vec.len() == 1 {
            let value_option = vec.first();
            return match value_option {
                Some(&DataType::List(ref l)) => Ok(Some(DataType::List(l.clone()))),
                Some(&DataType::Integer(n)) => Ok(Some(DataType::Integer(n))),
                Some(&DataType::Float(n)) => Ok(Some(DataType::Float(n))),
                Some(&DataType::Bool(b)) => Ok(Some(DataType::Bool(b))),
                Some(&DataType::Symbol(ref s)) => Ok(Some(DataType::Symbol(s.clone()))),
                Some(&DataType::String(ref s)) => Ok(Some(DataType::String(s.clone()))),
                Some(&DataType::Proc(ref p)) => Ok(Some(DataType::Proc(p.clone()))),
                Some(&DataType::Lambda(ref l)) => Ok(Some(DataType::Lambda(l.clone()))),
                Some(&DataType::Pair(ref p)) => Ok(Some(DataType::Pair((p.0.clone(), p.1.clone())))),
                None => { return Err("append function unknown argument type".into()); }
            };
        }

        //        let first_option = vec.first();
        let first_option = vec.get(0);
        let rest_option = if vec.len() > 1 { Some(&vec[1..]) } else { None };

        match first_option {
            Some(&DataType::List(ref l1)) => {
                let mut list = l1.clone();

                match rest_option {
                    Some(rest) => {
                        for item in rest.iter() {
                            match item {
                                &DataType::List(ref l2) => list.append(&mut l2.clone()),
                                &DataType::Integer(n) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Integer(n)))
                                        )
                                    ))
                                },
                                &DataType::Float(n) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Float(n)))
                                        )
                                    ))
                                },
                                &DataType::Bool(b) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Bool(b)))
                                        )
                                    ))
                                },
                                &DataType::Pair(ref p) => {
                                    list.push((*p.0).clone());
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             p.1.clone())
                                        )
                                    ))
                                },
                                &DataType::Symbol(ref s) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Symbol(s.clone())))
                                        )
                                    ))
                                },
                                &DataType::String(ref s) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::String(s.clone())))
                                        )
                                    ))
                                },
                                &DataType::Proc(ref p) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Proc(p.clone())))
                                        )
                                    ))
                                },
                                &DataType::Lambda(ref l) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Lambda(l.clone())))
                                        )
                                    ))
                                }
                            }
                        }
                    }
                    None => {
                        return Err("append function requires an argument of type 'list'".into());
                    }
                }

                Ok(Some(DataType::List(list.clone())))
            }
            Some(_) => { return Err("append function wrong type of the first argument".into()); }
            None => { return Err("append function unknown argument type".into()); }
        }
    }))));

    map.insert("apply".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, env: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "apply", vec);

        if vec.len() != 2 {
            return Err("apply function requires two arguments".into());
        }

        let s0 = vec.get(0);
        let s1 = vec.get(1);
        if let Some(&DataType::List(ref args)) = s1 {
            match s0 {
                Some(&DataType::Proc(ref f)) => {
                    f.call(args.clone(), env.clone()).and_then(|r| {
                        match r {
                            Some(data) => Ok(Some(data)),
                            None => Ok(None)
                        }
                    })
                }
                Some(&DataType::Lambda(ref p)) => {
                    debug!("first elm symbol - lambda: {:?}", p);
                    debug!("first elm symbol - procedure params: {:?}", p.params);
                    let proc_env = Env::child(p.env.clone());

                    for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                        debug!("first elm symbol - procedure params - name: {:?} value: {:?}", name_ref, value_ref);
                        if let (Some(&DataType::Symbol(ref name)), Some(value)) = (Some(name_ref), Some(value_ref)) {
                            proc_env.borrow().define(name.to_string(), value.clone());
                        } else {
                            return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                        }
                    }

                    debug!("proc_env: {:?}", proc_env);
                    return eval(Some((*p.body).clone()), proc_env);
                }
                Some(_) | None => Err("apply function unknown first argument type".into())
            }
        } else {
            return Err("apply function requires two arguments".into());
        }
    }))));

    // pre-defined commands
    map.insert("begin".to_string(), DataType::Proc(
        Function(
            Rc::new(|mut vec: Vec<DataType>, _: EnvRef| {
                debug!("Function - name: {:?} - Args: {:?}", "begin", vec);
                Ok(vec.pop().clone())
            })
        )
    ));

    map.insert("car".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "car", vec);
        if vec.len() != 1 {
            return Err("car function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("car function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::List(ref vec) => {
                let value = vec.first();
                if value.is_some() {
                    Ok(Some(DataType::from(value.unwrap().clone())))
                } else {
                    Err("car function requires a non-empty list".into())
                }
            }
            &DataType::Pair(ref p) => Ok(Some(*(p.0).clone())),
            _ => Err("car function requires an argument of type 'list' / 'pair'".into())
        }
    }))));

    map.insert("cdr".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "cdr", vec);
        if vec.len() != 1 {
            return Err("cdr function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("cdr function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::List(ref vec) => {
                if vec.len() > 0 {
                    Ok(Some(DataType::List((&vec[1..]).to_vec())))
                } else {
                    Err("cdr function requires a non-empty list".into())
                }
            },
            &DataType::Pair(ref p) => Ok(Some(*(p.1).clone())),
            _ => Err("cdr function requires an argument of type 'list'/ 'pair'".into())
        }
    }))));

    map.insert("cons".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "cons", vec);
        if vec.len() != 2 {
            return Err("cons function requires two argument only".into());
        }

        if let (Some(x), Some(y)) = (vec.get(0), vec.get(1)) {
            match y {
                &DataType::List(ref l) => {
                    let mut result :Vec<DataType> = vec![(*x).clone()];
                    result.extend(l.iter().map(|item| item.clone()));
                    Ok(Some(DataType::List(result)))
                },
                _ => {
                    Ok(Some(DataType::Pair(
                        (Box::new(x.clone()), Box::new(y.clone()))
                    )))
                }
            }
        } else {
            return Err("cons function unknown error".into())
        }
    }))));

    map.insert("length".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "length", vec);
        if vec.len() != 1 {
            return Err("length function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("length function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::List(ref vec) => Ok(Some(DataType::Integer(vec.len() as i64))),
            _ => Err("length function requires an argument of type 'list'".into())
        }
    }))));

    map.insert("list".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "list", vec);
        Ok(Some(DataType::List(vec)))
    }))));

    map.insert("list?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "list?", vec);
        if vec.len() != 1 {
            return Err("list? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("list? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::List(_) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));

    map.insert("null?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("null? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::List(ref v)) if v.is_empty() => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false))),
        }
    }))));

    map.insert("map".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, env: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "map", vec);
        if vec.len() != 2 {
            return Err("map function requires two argument only".into());
        }

        let value_option = vec.first();
        if value_option.is_none() {
            return Err("map function unknown argument type".into());
        }

        if let (Some(d), Some(&DataType::List(ref l))) = (vec.first(), vec.get(1)) {
            match d {
                &DataType::Proc(ref f) => {
                    let list = l.iter()
                        .map(|item| f.call(vec![item.clone()], env.clone()))
                        .flat_map(|x| x.ok())
                        .filter(|x| x.is_some())
                        .flat_map(|x| x)
                        .collect::<Vec<DataType>>();

                    Ok(Some(DataType::List(list)))
                },
                &DataType::Lambda(ref p) => {
                    let list = l.iter().map(|item| {
                        let proc_env = Env::child(p.env.clone());
                        let args = vec![item.clone()];
                        for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                            if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                proc_env.borrow().define(name.to_string(), value.clone());
                            } else {
                                return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                            }
                        }

                        debug!("proc_env: {:?}", proc_env);
                        eval(Some((*p.body).clone()), proc_env)
                    }).flat_map(|x| x.ok())
                        .filter(|x| x.is_some())
                        .flat_map(|x| x)
                        .collect::<Vec<DataType>>();

                    Ok(Some(DataType::List(list)))
                },
                _ => return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
            }
        } else {
            Err("syntax error".into())
        }
    }))));

    map.insert("max".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "max", vec);
        if !vec.iter().all(|x| x.is_number()) {
            return Err("wrong argument datatype".into());
        }
        let all_int = vec.iter().all(|x| x.is_integer());
        if all_int {
            let data = vec.iter().filter_map(|x| if let &DataType::Integer(i) = x { Some(i) } else { None }).max().unwrap();
            Ok(Some(DataType::Integer(data)))
        } else {
            let data = vec.iter().filter_map(|x| x.as_f64()).float_max();
            Ok(Some(DataType::Float(data)))
        }
    }))));

    map.insert("min".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "min", vec);
        if !vec.iter().all(|x| x.is_number()) {
            return Err("wrong argument datatype".into());
        }
        let all_int = vec.iter().all(|x| x.is_integer());
        if all_int {
            let data = vec.iter().filter_map(|x| if let &DataType::Integer(i) = x { Some(i) } else { None }).min().unwrap();
            Ok(Some(DataType::Integer(data)))
        } else {
            let data = vec.iter().filter_map(|x| x.as_f64()).float_min();
            Ok(Some(DataType::Float(data)))
        }
    }))));

    map.insert("not".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "not", vec);
        if vec.len() != 1 {
            return Err("not function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("not function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Bool(b) => Ok(Some(DataType::Bool(!b))),
            _ => Err("not function requires an argument of type 'boolean'".into())
        }
    }))));

    map.insert("number?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "number?", vec);
        if vec.len() != 1 {
            return Err("number? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("number? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Integer(_) => Ok(Some(DataType::Bool(true))),
            &DataType::Float(_) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));
    map.insert("pair?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "pair?", vec);
        if vec.len() != 1 {
            return Err("pair? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("pair? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Pair(_) => Ok(Some(DataType::Bool(true))),
            &DataType::List(ref v) => Ok(Some(DataType::Bool(!v.is_empty()))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));

    map.insert("print".to_string(), DataType::Proc(
        Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
            debug!("Function - name: {:?} - Args: {:?}", "print", vec);
            if vec.len() != 1 {
                return Err("print function requires one argument only".into());
            }

            let value_option = vec.first();
            if value_option.is_none() {
                return Err("unknown argument type".into());
            }
            println!("{}", datatype2str(value_option.unwrap()));
            //        print_fn(value_option.unwrap());
            Ok(None)
        }))));

    map.insert("procedure?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "procedure?", vec);
        if vec.len() != 1 {
            return Err("procedure? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("procedure? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Proc(_) => Ok(Some(DataType::Bool(true))),
            &DataType::Lambda(_) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));

    map.insert("string?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "string?", vec);
        if vec.len() != 1 {
            return Err("string? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("string? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::String(_) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));

    map.insert("symbol?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        debug!("Function - name: {:?} - Args: {:?}", "symbol?", vec);
        if vec.len() != 1 {
            return Err("symbol? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("symbol? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Symbol(_) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));

    //    debug!("map start");
    //    for (i, key) in map.keys().enumerate() {
    //        debug!("{} => {}", i + 1, key);
    //        match map.get(key) {
    //            Some(&DataType::Proc(ref f)) => {
    //                match f.call(vec![DataType::Integer(1), DataType::Integer(2), DataType::Float(5.1)]) {
    //                    Ok(result) => { debug!("Execution is good. Result: {:?}", result); }
    //                    Err(_) => { debug!("Execution is failed"); }
    //                }
    //            }
    //            Some(&ref o) => {
    //                debug!("{:?}", o);
    //            },
    //            None => {}
    //        }
    //    }
    //    debug!("map end");

    // --- R5RS equality predicates ---

    // eq? — identity comparison (same symbol, same number, same bool)
    map.insert("eq?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("eq? requires 2 arguments".into()); }
        let a = vec.get(0).unwrap();
        let b = vec.get(1).unwrap();
        Ok(Some(DataType::Bool(a == b)))
    }))));

    // eqv? — type-sensitive equality (Integer(1) != Float(1.0))
    map.insert("eqv?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("eqv? requires 2 arguments".into()); }
        let a = vec.get(0).unwrap();
        let b = vec.get(1).unwrap();
        // eqv? is type-sensitive: same type + same value
        let same = match (a, b) {
            (&DataType::Integer(x), &DataType::Integer(y)) => x == y,
            (&DataType::Float(x), &DataType::Float(y)) => x == y,
            (&DataType::Bool(x), &DataType::Bool(y)) => x == y,
            (&DataType::Symbol(ref x), &DataType::Symbol(ref y)) => x == y,
            (&DataType::String(ref x), &DataType::String(ref y)) => x == y,
            _ => false,
        };
        Ok(Some(DataType::Bool(same)))
    }))));

    // equal? — deep equality (lists compared element-wise)
    map.insert("equal?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("equal? requires 2 arguments".into()); }
        let a = vec.get(0).unwrap();
        let b = vec.get(1).unwrap();
        Ok(Some(DataType::Bool(a == b)))
    }))));

    // --- R5RS output ---

    // display — print without quotes (strings show raw, symbols show without ')
    map.insert("display".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("display requires 1 argument".into()); }
        let val = vec.get(0).unwrap();
        let s = match val {
            &DataType::String(ref s) => s.clone(),
            &DataType::Symbol(ref s) => s.clone(),
            &DataType::Bool(b) => if b { "#t".to_string() } else { "#f".to_string() },
            ref other => datatype2str(other),
        };
        print!("{}", s);
        Ok(None)
    }))));

    // newline — print a newline
    map.insert("newline".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if !vec.is_empty() { return Err("newline takes no arguments".into()); }
        println!();
        Ok(None)
    }))));

    // --- R5RS string operations ---

    map.insert("string-length".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("string-length requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::String(ref s)) => Ok(Some(DataType::Integer(s.len() as i64))),
            _ => Err("string-length requires a string".into()),
        }
    }))));

    map.insert("string-append".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        let mut result = String::new();
        for arg in &vec {
            match arg {
                &DataType::String(ref s) => result.push_str(s),
                _ => return Err("string-append requires string arguments".into()),
            }
        }
        Ok(Some(DataType::String(result)))
    }))));

    map.insert("string->symbol".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("string->symbol requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::String(ref s)) => Ok(Some(DataType::Symbol(s.clone()))),
            _ => Err("string->symbol requires a string".into()),
        }
    }))));

    map.insert("symbol->string".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("symbol->string requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Symbol(ref s)) => Ok(Some(DataType::String(s.clone()))),
            _ => Err("symbol->string requires a symbol".into()),
        }
    }))));

    // --- R5RS type/number predicates ---

    map.insert("boolean?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("boolean? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Bool(_)) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false))),
        }
    }))));

    map.insert("zero?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("zero? requires 1 argument".into()); }
        match vec.get(0) {
            Some(x) if x.is_number() => Ok(Some(DataType::Bool(x.as_f64().unwrap() == 0.0))),
            _ => Err("zero? requires a number".into()),
        }
    }))));

    map.insert("positive?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("positive? requires 1 argument".into()); }
        match vec.get(0) {
            Some(x) if x.is_number() => Ok(Some(DataType::Bool(x.as_f64().unwrap() > 0.0))),
            _ => Err("positive? requires a number".into()),
        }
    }))));

    map.insert("negative?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("negative? requires 1 argument".into()); }
        match vec.get(0) {
            Some(x) if x.is_number() => Ok(Some(DataType::Bool(x.as_f64().unwrap() < 0.0))),
            _ => Err("negative? requires a number".into()),
        }
    }))));

    map.insert("even?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("even? requires 1 argument".into()); }
        match vec.get(0) {
            Some(x) if x.is_number() => Ok(Some(DataType::Bool(x.as_f64().unwrap() as i64 % 2 == 0))),
            _ => Err("even? requires a number".into()),
        }
    }))));

    map.insert("odd?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("odd? requires 1 argument".into()); }
        match vec.get(0) {
            Some(x) if x.is_number() => Ok(Some(DataType::Bool(x.as_f64().unwrap() as i64 % 2 != 0))),
            _ => Err("odd? requires a number".into()),
        }
    }))));

    // --- R5RS integer division ---

    map.insert("modulo".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("modulo requires 2 arguments".into()); }
        if let (Some(a), Some(b)) = (vec.get(0), vec.get(1)) {
            if !a.is_number() || !b.is_number() { return Err("modulo requires numbers".into()); }
            let ai = a.as_f64().unwrap() as i64;
            let bi = b.as_f64().unwrap() as i64;
            if bi == 0 { return Err(SchemeError::DivisionByZero); }
            Ok(Some(DataType::Integer(ai % bi)))
        } else {
            Err("modulo requires numbers".into())
        }
    }))));

    map.insert("quotient".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("quotient requires 2 arguments".into()); }
        if let (Some(a), Some(b)) = (vec.get(0), vec.get(1)) {
            if !a.is_number() || !b.is_number() { return Err("quotient requires numbers".into()); }
            let ai = a.as_f64().unwrap() as i64;
            let bi = b.as_f64().unwrap() as i64;
            if bi == 0 { return Err(SchemeError::DivisionByZero); }
            Ok(Some(DataType::Integer(ai / bi)))
        } else {
            Err("quotient requires numbers".into())
        }
    }))));

    map.insert("remainder".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("remainder requires 2 arguments".into()); }
        if let (Some(a), Some(b)) = (vec.get(0), vec.get(1)) {
            if !a.is_number() || !b.is_number() { return Err("remainder requires numbers".into()); }
            let ai = a.as_f64().unwrap() as i64;
            let bi = b.as_f64().unwrap() as i64;
            if bi == 0 { return Err(SchemeError::DivisionByZero); }
            Ok(Some(DataType::Integer(ai % bi)))
        } else {
            Err("remainder requires numbers".into())
        }
    }))));

    // --- List utilities ---

    map.insert("reverse".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("reverse requires 1 argument".into()); }
        if let Some(DataType::List(ref l)) = vec.get(0) {
            Ok(Some(DataType::List(l.iter().rev().cloned().collect())))
        } else {
            Err("reverse requires a list".into())
        }
    }))));

    map.insert("list-ref".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("list-ref requires 2 arguments".into()); }
        if let (Some(DataType::List(ref l)), Some(idx)) = (vec.get(0), vec.get(1)) {
            let i = idx.as_f64().unwrap() as usize;
            if i < l.len() {
                Ok(Some(l[i].clone()))
            } else {
                Err("list-ref: index out of bounds".into())
            }
        } else {
            Err("list-ref requires a list and an index".into())
        }
    }))));

    map.insert("list-tail".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("list-tail requires 2 arguments".into()); }
        if let (Some(DataType::List(ref l)), Some(idx)) = (vec.get(0), vec.get(1)) {
            let i = idx.as_f64().unwrap() as usize;
            if i <= l.len() {
                Ok(Some(DataType::List(l[i..].to_vec())))
            } else {
                Err("list-tail: index out of bounds".into())
            }
        } else {
            Err("list-tail requires a list and an index".into())
        }
    }))));

    // member / memq / memv — find element in list, return tail from match
    let make_member = |cmp: fn(&DataType, &DataType) -> bool, name: &str| -> DataType {
        let name = name.to_string();
        DataType::Proc(Function(Rc::new(move |vec: Vec<DataType>, _: EnvRef| {
            if vec.len() != 2 { return Err(SchemeError::RuntimeError(format!("{} requires 2 arguments", name))); }
            if let (Some(obj), Some(DataType::List(ref l))) = (vec.get(0), vec.get(1)) {
                for (i, item) in l.iter().enumerate() {
                    if cmp(obj, item) {
                        return Ok(Some(DataType::List(l[i..].to_vec())));
                    }
                }
                Ok(Some(DataType::Bool(false)))
            } else {
                Err(SchemeError::RuntimeError(format!("{} requires an object and a list", name)))
            }
        })))
    };
    map.insert("member".to_string(), make_member(|a, b| a == b, "member"));
    map.insert("memq".to_string(), make_member(|a, b| {
        match (a, b) {
            (DataType::Symbol(ref x), DataType::Symbol(ref y)) => x == y,
            (DataType::Integer(x), DataType::Integer(y)) => x == y,
            (DataType::Bool(x), DataType::Bool(y)) => x == y,
            _ => false,
        }
    }, "memq"));
    map.insert("memv".to_string(), make_member(|a, b| {
        match (a, b) {
            (DataType::Symbol(ref x), DataType::Symbol(ref y)) => x == y,
            (DataType::Integer(x), DataType::Integer(y)) => x == y,
            (DataType::Float(x), DataType::Float(y)) => x == y,
            (DataType::Bool(x), DataType::Bool(y)) => x == y,
            _ => false,
        }
    }, "memv"));

    // assoc / assq / assv — find pair by key in association list
    let make_assoc = |cmp: fn(&DataType, &DataType) -> bool, name: &str| -> DataType {
        let name = name.to_string();
        DataType::Proc(Function(Rc::new(move |vec: Vec<DataType>, _: EnvRef| {
            if vec.len() != 2 { return Err(SchemeError::RuntimeError(format!("{} requires 2 arguments", name))); }
            if let (Some(key), Some(DataType::List(ref l))) = (vec.get(0), vec.get(1)) {
                for pair in l.iter() {
                    if let DataType::List(ref p) = pair {
                        if let Some(k) = p.get(0) {
                            if cmp(key, k) {
                                return Ok(Some(pair.clone()));
                            }
                        }
                    }
                }
                Ok(Some(DataType::Bool(false)))
            } else {
                Err(SchemeError::RuntimeError(format!("{} requires a key and an association list", name)))
            }
        })))
    };
    map.insert("assoc".to_string(), make_assoc(|a, b| a == b, "assoc"));
    map.insert("assq".to_string(), make_assoc(|a, b| {
        match (a, b) {
            (DataType::Symbol(ref x), DataType::Symbol(ref y)) => x == y,
            (DataType::Integer(x), DataType::Integer(y)) => x == y,
            (DataType::Bool(x), DataType::Bool(y)) => x == y,
            _ => false,
        }
    }, "assq"));
    map.insert("assv".to_string(), make_assoc(|a, b| {
        match (a, b) {
            (DataType::Symbol(ref x), DataType::Symbol(ref y)) => x == y,
            (DataType::Integer(x), DataType::Integer(y)) => x == y,
            (DataType::Float(x), DataType::Float(y)) => x == y,
            (DataType::Bool(x), DataType::Bool(y)) => x == y,
            _ => false,
        }
    }, "assv"));

    // --- String utilities ---

    map.insert("string=?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("string=? requires 2 arguments".into()); }
        if let (Some(DataType::String(ref a)), Some(DataType::String(ref b))) = (vec.get(0), vec.get(1)) {
            Ok(Some(DataType::Bool(a == b)))
        } else {
            Err("string=? requires strings".into())
        }
    }))));

    map.insert("string<?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("string<? requires 2 arguments".into()); }
        if let (Some(DataType::String(ref a)), Some(DataType::String(ref b))) = (vec.get(0), vec.get(1)) {
            Ok(Some(DataType::Bool(a < b)))
        } else {
            Err("string<? requires strings".into())
        }
    }))));

    map.insert("string>?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("string>? requires 2 arguments".into()); }
        if let (Some(DataType::String(ref a)), Some(DataType::String(ref b))) = (vec.get(0), vec.get(1)) {
            Ok(Some(DataType::Bool(a > b)))
        } else {
            Err("string>? requires strings".into())
        }
    }))));

    map.insert("substring".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() < 3 { return Err("substring requires 3 arguments".into()); }
        if let (Some(DataType::String(ref s)), Some(start), Some(end)) = (vec.get(0), vec.get(1), vec.get(2)) {
            let st = start.as_f64().unwrap() as usize;
            let en = end.as_f64().unwrap() as usize;
            if en > s.len() || st > en {
                return Err("substring: indices out of bounds".into());
            }
            Ok(Some(DataType::String(s[st..en].to_string())))
        } else {
            Err("substring requires a string and two indices".into())
        }
    }))));

    map.insert("string-ref".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("string-ref requires 2 arguments".into()); }
        if let (Some(DataType::String(ref s)), Some(idx)) = (vec.get(0), vec.get(1)) {
            let i = idx.as_f64().unwrap() as usize;
            if i < s.len() {
                Ok(Some(DataType::String(s[i..i+1].to_string())))
            } else {
                Err("string-ref: index out of bounds".into())
            }
        } else {
            Err("string-ref requires a string and an index".into())
        }
    }))));

    map.insert("string->list".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("string->list requires 1 argument".into()); }
        if let Some(DataType::String(ref s)) = vec.get(0) {
            // No char type — return list of 1-char strings
            Ok(Some(DataType::List(s.chars().map(|c| DataType::String(c.to_string())).collect())))
        } else {
            Err("string->list requires a string".into())
        }
    }))));

    map.insert("list->string".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("list->string requires 1 argument".into()); }
        if let Some(DataType::List(ref l)) = vec.get(0) {
            let mut result = String::new();
            for item in l.iter() {
                if let DataType::String(ref s) = item {
                    result.push_str(s);
                } else {
                    return Err("list->string requires a list of strings".into());
                }
            }
            Ok(Some(DataType::String(result)))
        } else {
            Err("list->string requires a list".into())
        }
    }))));

    map.insert("make-string".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() < 1 { return Err("make-string requires at least 1 argument".into()); }
        let n = vec.get(0).map(|d| d.as_f64().unwrap() as usize).unwrap_or(0);
        let fill = vec.get(1).and_then(|d| if let DataType::String(ref s) = d { s.chars().next() } else { None }).unwrap_or(' ');
        Ok(Some(DataType::String(fill.to_string().repeat(n))))
    }))));

    // --- Math functions ---

    map.insert("sqrt".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("sqrt requires 1 argument".into()); }
        if let Some(n) = vec.get(0) {
            if !n.is_number() { return Err("sqrt requires a number".into()); }
            Ok(Some(DataType::Float(n.as_f64().unwrap().sqrt())))
        } else {
            Err("sqrt requires a number".into())
        }
    }))));

    map.insert("expt".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 2 { return Err("expt requires 2 arguments".into()); }
        if let (Some(base), Some(exp)) = (vec.get(0), vec.get(1)) {
            if !base.is_number() || !exp.is_number() { return Err("expt requires numbers".into()); }
            let b = base.as_f64().unwrap();
            let e = exp.as_f64().unwrap();
            let result = b.powf(e);
            // Return Integer if both are integers and result is whole
            if base.is_integer() && exp.is_integer() && result.fract() == 0.0 && result.is_finite() {
                Ok(Some(DataType::Integer(result as i64)))
            } else {
                Ok(Some(DataType::Float(result)))
            }
        } else {
            Err("expt requires numbers".into())
        }
    }))));

    map.insert("floor".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("floor requires 1 argument".into()); }
        if let Some(n) = vec.get(0) {
            if !n.is_number() { return Err("floor requires a number".into()); }
            let f = n.as_f64().unwrap().floor();
            if n.is_integer() { Ok(Some(DataType::Integer(f as i64))) } else { Ok(Some(DataType::Integer(f as i64))) }
        } else { Err("floor requires a number".into()) }
    }))));

    map.insert("ceiling".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("ceiling requires 1 argument".into()); }
        if let Some(n) = vec.get(0) {
            if !n.is_number() { return Err("ceiling requires a number".into()); }
            Ok(Some(DataType::Integer(n.as_f64().unwrap().ceil() as i64)))
        } else { Err("ceiling requires a number".into()) }
    }))));

    map.insert("round".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("round requires 1 argument".into()); }
        if let Some(n) = vec.get(0) {
            if !n.is_number() { return Err("round requires a number".into()); }
            Ok(Some(DataType::Integer(n.as_f64().unwrap().round() as i64)))
        } else { Err("round requires a number".into()) }
    }))));

    map.insert("truncate".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("truncate requires 1 argument".into()); }
        if let Some(n) = vec.get(0) {
            if !n.is_number() { return Err("truncate requires a number".into()); }
            Ok(Some(DataType::Integer(n.as_f64().unwrap().trunc() as i64)))
        } else { Err("truncate requires a number".into()) }
    }))));

    map.insert("gcd".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.is_empty() { return Ok(Some(DataType::Integer(0))); }
        let mut result: i64 = 0;
        for arg in vec.iter() {
            if !arg.is_number() { return Err("gcd requires numbers".into()); }
            let mut a = arg.as_f64().unwrap() as i64;
            if a < 0 { a = -a; }
            let mut b = result;
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            result = a;
        }
        Ok(Some(DataType::Integer(result)))
    }))));

    map.insert("lcm".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.is_empty() { return Ok(Some(DataType::Integer(1))); }
        let mut result: i64 = 1;
        for arg in vec.iter() {
            if !arg.is_number() { return Err("lcm requires numbers".into()); }
            let a = arg.as_f64().unwrap() as i64;
            if a == 0 { return Ok(Some(DataType::Integer(0))); }
            let mut g = result;
            let mut b = a.abs();
            while b != 0 {
                let t = b;
                b = g % b;
                g = t;
            }
            result = (result.abs() / g) * a.abs();
        }
        Ok(Some(DataType::Integer(result)))
    }))));

    // --- Error handling ---

    map.insert("error".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.is_empty() { return Err(SchemeError::RuntimeError("error".to_string())); }
        let mut msg = String::new();
        if let Some(DataType::String(ref s)) = vec.get(0) {
            msg.push_str(s);
        } else {
            msg.push_str("error");
        }
        for arg in vec.iter().skip(1) {
            msg.push(' ');
            msg.push_str(&datatype2str(arg));
        }
        Err(SchemeError::RuntimeError(msg))
    }))));

    // --- for-each ---

    map.insert("for-each".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, env: EnvRef| {
        if vec.len() != 2 {
            return Err("for-each requires a procedure and a list".into());
        }
        if let (Some(d), Some(&DataType::List(ref l))) = (vec.first(), vec.get(1)) {
            match d {
                &DataType::Proc(ref f) => {
                    for item in l.iter() {
                        let _ = f.call(vec![item.clone()], env.clone())?;
                    }
                    Ok(None)
                },
                &DataType::Lambda(ref p) => {
                    for item in l.iter() {
                        let proc_env = Env::child(p.env.clone());
                        let args = vec![item.clone()];
                        for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                            if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                proc_env.borrow().define(name.to_string(), value.clone());
                            } else {
                                return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                            }
                        }
                        eval(Some((*p.body).clone()), proc_env)?;
                    }
                    Ok(None)
                },
                _ => Err("for-each: first argument must be a procedure".into())
            }
        } else {
            Err("for-each: second argument must be a list".into())
        }
    }))));

    // --- car/cdr compositions ---

    macro_rules! car_cdr_comp {
        ($name:expr, $body:expr) => {
            map.insert($name.to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
                if vec.len() != 1 { return Err(SchemeError::RuntimeError(format!("{} requires exactly one argument", $name))); }
                $body(&vec[0]).map(Some).ok_or_else(|| SchemeError::RuntimeError(format!("{}: not a pair or empty list", $name)))
            }))));
        }
    }

    car_cdr_comp!("caar", |x: &DataType| car(x).and_then(|d| car(&d)));
    car_cdr_comp!("cadr", |x: &DataType| cdr(x).and_then(|d| car(&d)));
    car_cdr_comp!("cdar", |x: &DataType| car(x).and_then(|d| cdr(&d)));
    car_cdr_comp!("cddr", |x: &DataType| cdr(x).and_then(|d| cdr(&d)));
    car_cdr_comp!("caaar", |x: &DataType| car(x).and_then(|d| car(&d)).and_then(|d| car(&d)));
    car_cdr_comp!("caadr", |x: &DataType| cdr(x).and_then(|d| car(&d)).and_then(|d| car(&d)));
    car_cdr_comp!("cadar", |x: &DataType| car(x).and_then(|d| cdr(&d)).and_then(|d| car(&d)));
    car_cdr_comp!("caddr", |x: &DataType| cdr(x).and_then(|d| cdr(&d)).and_then(|d| car(&d)));
    car_cdr_comp!("cdaar", |x: &DataType| car(x).and_then(|d| car(&d)).and_then(|d| cdr(&d)));
    car_cdr_comp!("cdadr", |x: &DataType| cdr(x).and_then(|d| car(&d)).and_then(|d| cdr(&d)));
    car_cdr_comp!("cddar", |x: &DataType| car(x).and_then(|d| cdr(&d)).and_then(|d| cdr(&d)));
    car_cdr_comp!("cdddr", |x: &DataType| cdr(x).and_then(|d| cdr(&d)).and_then(|d| cdr(&d)));
    car_cdr_comp!("cadddr", |x: &DataType| cdr(x).and_then(|d| cdr(&d)).and_then(|d| cdr(&d)).and_then(|d| car(&d)));

    // --- Numeric type predicates ---

    map.insert("integer?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("integer? requires one argument".into()); }
        Ok(Some(DataType::Bool(matches!(vec[0], DataType::Integer(_)))))
    }))));

    map.insert("real?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("real? requires one argument".into()); }
        Ok(Some(DataType::Bool(vec[0].is_number())))
    }))));

    // --- string->number / number->string ---

    map.insert("string->number".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("string->number requires one argument".into()); }
        if let DataType::String(ref s) = vec[0] {
            if let Ok(i) = s.parse::<i64>() {
                return Ok(Some(DataType::Integer(i)));
            }
            if let Ok(f) = s.parse::<f64>() {
                return Ok(Some(DataType::Float(f)));
            }
            // R5RS: returns #f if not a number
            Ok(Some(DataType::Bool(false)))
        } else {
            Err("string->number requires a string argument".into())
        }
    }))));

    map.insert("number->string".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() < 1 || vec.len() > 2 { return Err("number->string requires one or two arguments".into()); }
        if !vec[0].is_number() { return Err("number->string: first argument must be a number".into()); }
        let radix = if vec.len() == 2 {
            if let DataType::Integer(r) = vec[1] { r } else { return Err("number->string: radix must be an integer".into()); }
        } else { 10 };
        let s = match vec[0] {
            DataType::Integer(i) => match radix {
                2 => format!("{:b}", i),
                8 => format!("{:o}", i),
                16 => format!("{:x}", i),
                10 | _ => i.to_string(),
            },
            DataType::Float(f) => {
                if radix != 10 { format!("{:x}", f as i64) } else { f.to_string() }
            },
            _ => return Err("number->string: not a number".into()),
        };
        Ok(Some(DataType::String(s)))
    }))));

    // --- string-copy ---

    map.insert("string-copy".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() < 1 || vec.len() > 3 { return Err("string-copy requires 1-3 arguments".into()); }
        if let DataType::String(ref s) = vec[0] {
            let start = if vec.len() >= 2 {
                if let DataType::Integer(i) = vec[1] { i as usize } else { return Err("string-copy: start must be integer".into()); }
            } else { 0 };
            let end = if vec.len() >= 3 {
                if let DataType::Integer(i) = vec[2] { i as usize } else { return Err("string-copy: end must be integer".into()); }
            } else { s.len() };
            if start > end || end > s.len() {
                return Err("string-copy: indices out of range".into());
            }
            Ok(Some(DataType::String(s[start..end].to_string())))
        } else {
            Err("string-copy: first argument must be a string".into())
        }
    }))));

    // --- assert ---

    map.insert("assert".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() != 1 { return Err("assert requires one argument".into()); }
        match &vec[0] {
            DataType::Bool(true) => Ok(None),
            DataType::Bool(false) => Err(SchemeError::RuntimeError("assertion failed".to_string())),
            _ => {
                // R5RS: anything not #f is true
                Ok(None)
            }
        }
    }))));

    // --- Transcendental math functions ---

    macro_rules! transcendental {
        ($name:expr, $f:expr) => {
            map.insert($name.to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
                if vec.len() != 1 { return Err(SchemeError::RuntimeError(format!("{} requires one argument", $name))); }
                if !vec[0].is_number() { return Err(SchemeError::RuntimeError(format!("{} requires a number", $name))); }
                let x = vec[0].as_f64().unwrap();
                Ok(Some(DataType::Float($f(x))))
            }))));
        }
    }

    transcendental!("exp", f64::exp);
    transcendental!("log", f64::ln);
    transcendental!("sin", f64::sin);
    transcendental!("cos", f64::cos);
    transcendental!("tan", f64::tan);
    transcendental!("asin", f64::asin);
    transcendental!("acos", f64::acos);

    map.insert("atan".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: EnvRef| {
        if vec.len() < 1 || vec.len() > 2 { return Err("atan requires 1-2 arguments".into()); }
        if !vec.iter().all(|x| x.is_number()) { return Err("atan requires numbers".into()); }
        let y = vec[0].as_f64().unwrap();
        if vec.len() == 1 {
            Ok(Some(DataType::Float(y.atan())))
        } else {
            let x = vec[1].as_f64().unwrap();
            Ok(Some(DataType::Float(y.atan2(x))))
        }
    }))));

    // --- filter (not R5RS but commonly expected) ---

    map.insert("filter".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, env: EnvRef| {
        if vec.len() != 2 { return Err("filter requires a predicate and a list".into()); }
        if let (Some(d), Some(&DataType::List(ref l))) = (vec.first(), vec.get(1)) {
            let mut result = Vec::new();
            match d {
                &DataType::Proc(ref f) => {
                    for item in l.iter() {
                        if let Ok(Some(DataType::Bool(true))) = f.call(vec![item.clone()], env.clone()) {
                            result.push(item.clone());
                        }
                    }
                },
                &DataType::Lambda(ref p) => {
                    for item in l.iter() {
                        let proc_env = Env::child(p.env.clone());
                        let args = vec![item.clone()];
                        for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                            if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                proc_env.borrow().define(name.to_string(), value.clone());
                            } else {
                                return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                            }
                        }
                        if let Ok(Some(DataType::Bool(true))) = eval(Some((*p.body).clone()), proc_env) {
                            result.push(item.clone());
                        }
                    }
                },
                _ => return Err("filter: first argument must be a procedure".into())
            }
            Ok(Some(DataType::List(result)))
        } else {
            Err("filter: second argument must be a list".into())
        }
    }))));

    return map;
}
