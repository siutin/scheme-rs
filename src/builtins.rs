use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use log::debug;

use crate::types::{Function, DataType, FloatIterExt};
use crate::env::Env;
use crate::eval::{eval, datatype2str};
use crate::SchemeError;

#[macro_export]
macro_rules! define_comparison {
    ($proc:ident, $name:pat, $func:expr) => {
        let $proc = DataType::Proc(Function( Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
                debug!("Function - name: {:?} - Args: {:?}", stringify!($name), vec);
                if vec.len() != 2 {
                    return Err("function requires 2 arguments only".into());
                }
                let a = vec.get(0);
                let b = vec.get(1);

                if let (Some(&DataType::Number(ref a0)), Some(&DataType::Number(ref b0))) = (a, b) {
                    let a1: f64 = a0.clone().into();
                    let b1: f64 = b0.clone().into();
                    let desc = format!("{} {} {}", a1, stringify!($name), b1);
                    debug!("Description: {}", desc);
                    Ok(Some(DataType::Bool($func(a1, b1))))
                } else {
                    return Err("wrong argument datatype".into());
                }

            })));
    };
}

pub fn setup() -> HashMap<String, DataType> {
    let mut map = HashMap::new();
    map.insert("pi".to_string(), DataType::Number(std::f64::consts::PI));

    map.insert("+".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "+", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });
        if !is_numbers {
            return Err("wrong argument datatype".into());
        }

        let desc = vec.iter().map(|&ref x|
            match x {
                &DataType::Number(f) => f.to_string(),
                _ => String::new(),
            }
        ).collect::<Vec<String>>().join(" + ");
        debug!("Description: {}", desc);
        let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });
        let data: f64 = numbers.map(|x| {
            let y: f64 = x.clone().into();
            y
        }).sum();
        Ok(Some(DataType::Number(data)))
    }))));

    map.insert("-".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "-", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });

        if !is_numbers {
            return Err("wrong argument datatype".into());
        }

        if vec.is_empty() {
            return Err("- function requires at least one argument".into());
        }

        let numbers: Vec<f64> = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } })
            .map(|x| x.into())
            .collect();

        let value: f64 = if numbers.len() == 1 {
            -numbers[0]
        } else {
            let first = numbers[0];
            numbers[1..].iter().fold(first, |acc, x| acc - x)
        };
        Ok(Some(DataType::Number(value)))

    }))));

    map.insert("*".to_string(), DataType::Proc(
        Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
            debug!("Function - name: {:?} - Args: {:?}", "*", vec);
            let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });
            if !is_numbers {
                return Err("wrong argument datatype".into());
            }

            let desc = vec.iter().map(|&ref x|
                match x {
                    &DataType::Number(f) => f.to_string(),
                    _ => String::new(),
                }
            ).collect::<Vec<String>>().join(" x ");
            debug!("Description: {}", desc);

            let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });
            let data: f64 = numbers.map(|x| {
                let y: f64 = x.clone().into();
                y
            }).product();
            Ok(Some(DataType::Number(data)))
        }))));

    map.insert("/".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "/", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });

        if !is_numbers {
            return Err("wrong argument datatype".into());
        }

        if vec.is_empty() {
            return Err("/ function requires at least one argument".into());
        }

        let numbers: Vec<f64> = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } })
            .map(|x| x.into())
            .collect();

        // Check for division by zero in any divisor
        if numbers.len() > 1 && numbers[1..].iter().any(|&x| x == 0.0) {
            return Err(SchemeError::DivisionByZero);
        }

        let value: f64 = if numbers.len() == 1 {
            1.0 / numbers[0]
        } else {
            let first = numbers[0];
            numbers[1..].iter().fold(first, |acc, x| acc / x)
        };
        Ok(Some(DataType::Number(value)))
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

    map.insert("abs".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "abs", vec);
        if vec.len() != 1 {
            return Err("abs function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("abs function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Number(f) => Ok(Some(DataType::Number(f.abs()))),
            _ => Err("abs function requires an argument of type 'number'".into())
        }
    }))));

    map.insert("append".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "append", vec);

        if vec.is_empty() {
            return Ok(Some(DataType::List(vec![])));
        }

        if vec.len() == 1 {
            let value_option = vec.first();
            return match value_option {
                Some(&DataType::List(ref l)) => Ok(Some(DataType::List(l.clone()))),
                Some(&DataType::Number(n)) => Ok(Some(DataType::Number(n))),
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
                                &DataType::Number(n) => {
                                    return Ok(Some(
                                        DataType::Pair(
                                            (Box::new(DataType::List(list.clone())),
                                             Box::new(DataType::Number(n)))
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

    map.insert("apply".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, env: Rc<RefCell<Env>>| {
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
                    let procedure_local = Box::new(RefCell::new(HashMap::new()));

                    for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                        debug!("first elm symbol - procedure params - name: {:?} value: {:?}", name_ref, value_ref);
                        if let (Some(&DataType::Symbol(ref name)), Some(value)) = (Some(name_ref), Some(value_ref)) {
                            procedure_local.borrow_mut().insert(name.to_string(), value.clone());
                        } else {
                            return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                        }
                    }

                    let proc_env = Env {
                        local: procedure_local,
                        parent: Some(Box::new(p.env.clone()))
                    };

                    debug!("proc_env: {:?}", proc_env);
                    return eval(Some(p.body.clone()), Rc::new(RefCell::new(proc_env)));
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
            Rc::new(|mut vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
                debug!("Function - name: {:?} - Args: {:?}", "begin", vec);
                Ok(vec.pop().clone())
            })
        )
    ));

    map.insert("car".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("cdr".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("cons".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("length".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "length", vec);
        if vec.len() != 1 {
            return Err("length function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("length function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::List(ref vec) => Ok(Some(DataType::Number(vec.len() as f64))),
            _ => Err("length function requires an argument of type 'list'".into())
        }
    }))));

    map.insert("list".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "list", vec);
        Ok(Some(DataType::List(vec)))
    }))));

    map.insert("list?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("map".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, env: Rc<RefCell<Env>>| {
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
                        let procedure_local = Box::new(RefCell::new(HashMap::new()));
                        let args = vec![item.clone()];
                        for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                            if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                procedure_local.borrow_mut().insert(name.to_string(), value.clone());
                            } else {
                                return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                            }
                        }

                        let proc_env = Env {
                            local: procedure_local,
                            parent: Some(Box::new(p.env.clone()))
                        };

                        debug!("proc_env: {:?}", proc_env);
                        eval(Some(p.body.clone()), Rc::new(RefCell::new(proc_env)))
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

    map.insert("max".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "max", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });
        if !is_numbers {
            return Err("wrong argument datatype".into());
        }
        let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });
        let data = numbers.map(|x| {
            let y: f64 = x.clone().into();
            y
        }).float_max();
        Ok(Some(DataType::Number(data.into())))
    }))));

    map.insert("min".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "min", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });
        if !is_numbers {
            return Err("wrong argument datatype".into());
        }
        let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });

        let data = numbers.map(|x| {
            let y: f64 = x.clone().into();
            y
        }).float_min();
        Ok(Some(DataType::Number(data.into())))
    }))));

    map.insert("not".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("number?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        debug!("Function - name: {:?} - Args: {:?}", "number?", vec);
        if vec.len() != 1 {
            return Err("number? function requires one argument only".into());
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("number? function unknown argument type".into());
        }
        match value_option.unwrap() {
            &DataType::Number(_) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));
    map.insert("pair?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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
            _ => Ok(Some(DataType::Bool(false)))
        }
    }))));

    map.insert("print".to_string(), DataType::Proc(
        Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("procedure?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("string?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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

    map.insert("symbol?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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
    map.insert("eq?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 2 { return Err("eq? requires 2 arguments".into()); }
        let a = vec.get(0).unwrap();
        let b = vec.get(1).unwrap();
        Ok(Some(DataType::Bool(a == b)))
    }))));

    // eqv? — same as eq? for our purposes (numbers, symbols, bools)
    map.insert("eqv?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 2 { return Err("eqv? requires 2 arguments".into()); }
        let a = vec.get(0).unwrap();
        let b = vec.get(1).unwrap();
        Ok(Some(DataType::Bool(a == b)))
    }))));

    // equal? — deep equality (lists compared element-wise)
    map.insert("equal?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 2 { return Err("equal? requires 2 arguments".into()); }
        let a = vec.get(0).unwrap();
        let b = vec.get(1).unwrap();
        Ok(Some(DataType::Bool(a == b)))
    }))));

    // --- R5RS output ---

    // display — print without quotes (strings show raw, symbols show without ')
    map.insert("display".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
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
    map.insert("newline".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if !vec.is_empty() { return Err("newline takes no arguments".into()); }
        println!();
        Ok(None)
    }))));

    // --- R5RS string operations ---

    map.insert("string-length".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("string-length requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::String(ref s)) => Ok(Some(DataType::Number(s.len() as f64))),
            _ => Err("string-length requires a string".into()),
        }
    }))));

    map.insert("string-append".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        let mut result = String::new();
        for arg in &vec {
            match arg {
                &DataType::String(ref s) => result.push_str(s),
                _ => return Err("string-append requires string arguments".into()),
            }
        }
        Ok(Some(DataType::String(result)))
    }))));

    map.insert("string->symbol".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("string->symbol requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::String(ref s)) => Ok(Some(DataType::Symbol(s.clone()))),
            _ => Err("string->symbol requires a string".into()),
        }
    }))));

    map.insert("symbol->string".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("symbol->string requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Symbol(ref s)) => Ok(Some(DataType::String(s.clone()))),
            _ => Err("symbol->string requires a symbol".into()),
        }
    }))));

    // --- R5RS type/number predicates ---

    map.insert("boolean?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("boolean? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Bool(_)) => Ok(Some(DataType::Bool(true))),
            _ => Ok(Some(DataType::Bool(false))),
        }
    }))));

    map.insert("zero?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("zero? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Number(n)) => Ok(Some(DataType::Bool(n == 0.0))),
            _ => Err("zero? requires a number".into()),
        }
    }))));

    map.insert("positive?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("positive? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Number(n)) => Ok(Some(DataType::Bool(n > 0.0))),
            _ => Err("positive? requires a number".into()),
        }
    }))));

    map.insert("negative?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("negative? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Number(n)) => Ok(Some(DataType::Bool(n < 0.0))),
            _ => Err("negative? requires a number".into()),
        }
    }))));

    map.insert("even?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("even? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Number(n)) => Ok(Some(DataType::Bool(n as i64 % 2 == 0))),
            _ => Err("even? requires a number".into()),
        }
    }))));

    map.insert("odd?".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 1 { return Err("odd? requires 1 argument".into()); }
        match vec.get(0) {
            Some(&DataType::Number(n)) => Ok(Some(DataType::Bool(n as i64 % 2 != 0))),
            _ => Err("odd? requires a number".into()),
        }
    }))));

    // --- R5RS integer division ---

    map.insert("modulo".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 2 { return Err("modulo requires 2 arguments".into()); }
        if let (Some(&DataType::Number(a)), Some(&DataType::Number(b))) = (vec.get(0), vec.get(1)) {
            if b == 0.0 { return Err(SchemeError::DivisionByZero); }
            Ok(Some(DataType::Number((a as i64 % b as i64) as f64)))
        } else {
            Err("modulo requires numbers".into())
        }
    }))));

    map.insert("quotient".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 2 { return Err("quotient requires 2 arguments".into()); }
        if let (Some(&DataType::Number(a)), Some(&DataType::Number(b))) = (vec.get(0), vec.get(1)) {
            if b == 0.0 { return Err(SchemeError::DivisionByZero); }
            Ok(Some(DataType::Number((a as i64 / b as i64) as f64)))
        } else {
            Err("quotient requires numbers".into())
        }
    }))));

    map.insert("remainder".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>, _: Rc<RefCell<Env>>| {
        if vec.len() != 2 { return Err("remainder requires 2 arguments".into()); }
        if let (Some(&DataType::Number(a)), Some(&DataType::Number(b))) = (vec.get(0), vec.get(1)) {
            if b == 0.0 { return Err(SchemeError::DivisionByZero); }
            Ok(Some(DataType::Number((a as i64 % b as i64) as f64)))
        } else {
            Err("remainder requires numbers".into())
        }
    }))));

    return map;
}
