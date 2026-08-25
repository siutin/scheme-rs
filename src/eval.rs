use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use log::debug;

use crate::types::{AST, Procedure, Function, DataType};
use crate::env::Env;
use crate::SchemeError;

pub fn eval(ast_option: Option<AST>, env: Rc<RefCell<Env>>) -> Result<Option<DataType>, SchemeError> {
    debug!("eval");
    debug!("{:?}", ast_option);
    match ast_option.clone() {
        Some(AST::Symbol(s)) => {
            debug!("ast is a symbol: {:?}", s);
            if s.starts_with("#") {
                if s.len() != 2 {
                    return Err("syntax error".into());
                }
                let c_option = s.chars().nth(1);
                if let Some('t') = c_option {
                    Ok(Some(DataType::Bool(true)))
                } else if let Some('f') = c_option {
                    Ok(Some(DataType::Bool(false)))
                } else {
                    Err("syntax error".into())
                }
            } else if s.len() > 1 && s.starts_with("'") {
                let slice = &s[1..s.len()];
                Ok(Some(DataType::Symbol(slice.to_string())))
            } else if s.starts_with("\"") && s.ends_with("\"") {
                Ok(Some(DataType::String((&s[1..s.len() - 1]).to_string())))
            } else {
                match env.borrow().get(&s) {
                    Some(data) => Ok(Some(data)),
                    None => Err("symbol is not defined.".into())
                }
            }
        }
        Some(AST::Children(list)) => {
            debug!("ast is a children: {:?}", list);

            if list.is_empty() {
                return Err("syntax error".into());
            }

            let s0 = list.get(0);
            let s1 = list.get(1);
            let s2 = list.get(2);
            let s3 = list.get(3);

            if let Some(&AST::Symbol(ref s0)) = s0 {
                match s0.as_str() {
                    "quote" => {
                        debug!("quote-expression");
                        let s1_option = list.get(1);

                        match s1_option {
                            Some(ref ast) => {
                                match ast2datatype(ast) {
                                    Ok(data) => Ok(Some(data)),
                                    Err(e) => { return Err(e); }
                                }
                            }
                            None => { return Err("wrong number of parts".into()); }
                        }
                    }
                    "if" => {
                        debug!("if-expression");
                        if let (Some(&ref cond), Some(&ref conseq), Some(&ref alt)) = (s1, s2, s3) {
                            match eval(Some(cond.clone()), env.clone()) {
                                Ok(Some(DataType::Bool(b))) => {
                                    match b {
                                        true => eval(Some(conseq.clone()), env.clone()),
                                        false => eval(Some(alt.clone()), env.clone())
                                    }
                                }
                                Ok(_) => { return Err("syntax error".into()); }
                                Err(e) => { return Err(e); }
                            }
                        } else {
                            return Err("wrong syntax for if expression".into());
                        }
                    }
                    "define" => {
                        if let (Some(&AST::Symbol(ref s1)), Some(&ref a2)) = (s1, s2) {
                            match a2.clone() {
                                AST::Integer(i) => {
                                    let env_borrow_mut = env.borrow_mut();
                                    env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Number(i as f64));
                                }
                                AST::Float(f) => {
                                    let env_borrow_mut = env.borrow_mut();
                                    env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Number(f));
                                }
                                AST::Symbol(ref s) => {
                                    if s.len() > 1 && s.starts_with("#") {
                                        let c_option = s.chars().nth(1);
                                        if let Some('t') = c_option {
                                            let env_borrow_mut = env.borrow_mut();
                                            env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Bool(true));
                                        } else if let Some('f') = c_option {
                                            let env_borrow_mut = env.borrow_mut();
                                            env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Bool(false));
                                        } else {
                                            return Err("syntax error".into());
                                        }
                                    } else if s.starts_with("\"") && s.ends_with("\"") {
                                        let env_borrow_mut = env.borrow_mut();
                                        env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::String((&s[1..s.len() - 1]).to_string()));
                                    } else {
                                        let data_option = env.borrow().get(&s);
                                        if let Some(data) = data_option {
                                            let env_borrow_mut = env.borrow_mut();
                                            env_borrow_mut.local.borrow_mut().insert(s1.clone(), data);
                                        } else {
                                            return Err("symbol is not defined".into());
                                        }
                                    }
                                }
                                AST::Children(ref v) => {
                                    debug!("children: {:?}", v);

                                    let data_option = eval(Some(a2.clone()), env.clone());
                                    if let Ok(Some(DataType::Lambda(ref p))) = data_option {
                                        let env_borrow_mut = env.borrow_mut();
                                        env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Lambda(p.clone()));
                                    } else if let Ok(Some(DataType::List(ref v))) = data_option {
                                        let env_borrow_mut = env.borrow_mut();
                                        env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::List(v.clone()));
                                    } else if let Err(e) = data_option {
                                        return Err(e);
                                    }
                                }
                            }
                            return Ok(None);
                        }
                        return Err("wrong syntax for define expression".into());
                    }
                    "lambda" => {
                        debug!("lambda-expression");
                        if let (Some(&AST::Children(ref args)), Some(body_ast)) = (s1, s2) {
                            debug!("ENV: {:?}", env);
                            debug!("args: {:?}", args);
                            debug!("body: {:?}", body_ast);

                            // convert args AST to Datatype symbol
                            let args_result: Result<Vec<_>, SchemeError> = args.iter().map(|ref arg|
                                match arg {
                                    &&AST::Symbol(ref arg_string) => Ok(DataType::Symbol(arg_string.to_string())),
                                    _ => Err("lambda argument must be a symbol".into())
                                }
                            ).collect();

                            if let Result::Err(ref e) = args_result { return Err(e.clone()); }

                            let args_meta = args_result.unwrap().iter()
                                .map(|ref mut x| x.clone())
                                .collect::<Vec<DataType>>();

                            let local = Box::new(RefCell::new(HashMap::new()));
                            let parent_env_box = Box::new(env.clone());
                            let procedure_env = Env {
                                local,
                                parent: Some(parent_env_box)
                            };

                            debug!("procedure_env: {:?}", procedure_env);
                            // Body can be a single expression or a list of expressions.
                            // Store as-is; eval handles both AST::Children and single AST nodes.
                            let procedure = Procedure {
                                body: body_ast.clone(),
                                params: args_meta,
                                env: Rc::new(RefCell::new(procedure_env))
                            };
                            debug!("procedure: {:?}", procedure);

                            Ok(Some(DataType::Lambda(procedure)))
                        } else {
                            Err("syntax error".into())
                        }
                    }
                    "let" => {
                        // (let ((var init) ...) body...)
                        if let (Some(&AST::Children(ref bindings)), Some(body_ast)) = (s1, s2) {
                            let mut local = HashMap::new();
                            for binding in bindings.iter() {
                                if let AST::Children(ref pair) = binding {
                                    if let (Some(&AST::Symbol(ref name)), Some(init)) = (pair.get(0), pair.get(1)) {
                                        match eval(Some(init.clone()), env.clone()) {
                                            Ok(Some(val)) => { local.insert(name.clone(), val); }
                                            Ok(None) => { local.insert(name.clone(), DataType::Bool(false)); }
                                            Err(e) => return Err(e),
                                        }
                                    } else {
                                        return Err("let binding must be (name init)".into());
                                    }
                                } else {
                                    return Err("let bindings must be a list of (name init) pairs".into());
                                }
                            }
                            let let_env = Env {
                                local: Box::new(RefCell::new(local)),
                                parent: Some(Box::new(env.clone())),
                            };
                            // Body can be a single expression or multiple; wrap in Children if needed
                            let body = match body_ast {
                                AST::Children(ref v) if v.len() == 1 => v[0].clone(),
                                AST::Children(_) => body_ast.clone(),
                                _ => body_ast.clone(),
                            };
                            eval(Some(body), Rc::new(RefCell::new(let_env)))
                        } else {
                            Err("let requires bindings and a body".into())
                        }
                    }
                    "cond" => {
                        // (cond (test expr...) ... (else expr...))
                        let mut result = Ok(Some(DataType::Bool(false)));
                        for i in 1..list.len() {
                            if let Some(&AST::Children(ref clause)) = list.get(i) {
                                let test = clause.get(0);
                                let body = clause.get(1);
                                // Check for else clause
                                let is_else = match test {
                                    Some(&AST::Symbol(ref s)) if s == "else" => true,
                                    _ => false,
                                };
                                if is_else {
                                    if let Some(body_ast) = body {
                                        result = eval(Some(body_ast.clone()), env.clone());
                                    }
                                    break;
                                }
                                if let Some(test_ast) = test {
                                    match eval(Some(test_ast.clone()), env.clone()) {
                                        Ok(Some(DataType::Bool(true))) => {
                                            if let Some(body_ast) = body {
                                                result = eval(Some(body_ast.clone()), env.clone());
                                            }
                                            break;
                                        }
                                        Ok(Some(DataType::Bool(false))) | Ok(None) => continue,
                                        Ok(_) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                            }
                        }
                        result
                    }
                    _ => {
                        debug!("Some(AST::Symbol) but not define");
                        debug!("proc_key : {}", s0);
                        debug!("ENV: {:?}", env);

                        let mut data_option = match env.borrow().get(s0) {
                            Some(d) => Some(d.clone()),
                            None => None
                        };

                        debug!("data_option: {:?}", data_option);

                        match data_option {
                            Some(DataType::Proc(ref f)) => {
                                let slice = &list[1..list.len()];
                                execute(f, slice, env)
                            }
                            Some(DataType::Lambda(ref mut p)) => {
                                debug!("first elm symbol - lambda: {:?}", p);
                                let slice = &list[1..list.len()];
                                match prepare_arguments(slice, env.clone()) {
                                    Ok(args) => {
                                        debug!("first elm symbol - procedure params: {:?}", p.params);
                                        let procedure_local = Box::new(RefCell::new(HashMap::new()));

                                        for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                                            debug!("first elm symbol - procedure params - name: {:?} value: {:?}", name_ref, value_ref);
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
                                        return eval(Some(p.body.clone()), Rc::new(RefCell::new(proc_env)));
                                    }
                                    Err(e) => return Err(e)
                                }
                            }
                            Some(_) | None => Err("symbol is not defined.".into())
                        }
                    }
                }
            } else {
                debug!("first ast is not a symbol");
                debug!("proc_key : {:?}", s0);

                let s0_option = list.get(0);
                let rest_option = if list.len() > 1 { Some(&list[1..]) } else { None };

                if let Some(&AST::Children(_)) = s0_option {
                    match eval(Some(list.first().unwrap().clone()), env.clone()) {
                        Ok(Some(DataType::Proc(ref f))) => {
                            debug!("first elm function - function: {:?}", f);
                            match rest_option {
                                Some(rest) => execute(f, rest, env),
                                None => execute(f, &vec![], env)
                            }
                        }
                        Ok(Some(DataType::Lambda(ref mut p))) => {
                            debug!("first elm lambda - lambda: {:?} - procedure params: {:?}", p, p.params);
                            let proc_env = match rest_option {
                                Some(rest) => {
                                    match prepare_arguments(rest, env.clone()) {
                                        Ok(args) => {
                                            let procedure_local = Box::new(RefCell::new(HashMap::new()));
                                            for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                                                debug!("first elm lambda - procedure params - name: {:?} value: {:?}", name_ref, value_ref);
                                                if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                                    procedure_local.borrow_mut().insert(name.to_string(), value.clone());
                                                } else {
                                                    return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                                                }
                                            }
                                            Env {
                                                local: procedure_local,
                                                parent: Some(Box::new(p.env.clone()))
                                            }
                                        }
                                        Err(e) => return Err(e)
                                    }
                                }
                                None => {
                                    Env {
                                        local: Box::new(RefCell::new(HashMap::new())),
                                        parent: Some(Box::new(p.env.clone()))
                                    }
                                }
                            };
                            debug!("proc_env: {:?}", proc_env);
                            return eval(Some(p.body.clone()), Rc::new(RefCell::new(proc_env)));
                        }
                        Ok(_) => { return Err("unsupported data type on first element".into()); }
                        Err(e) => { return Err(e); }
                    }
                } else {
                    return Err("syntax error".into());
                }
            }
        }
        Some(_) | None => {
            debug!("ast is not a symbol/children");
            let data = match ast_option {
                Some(AST::Integer(i)) => Some(DataType::Number(i as f64)),
                Some(AST::Float(f)) => Some(DataType::Number(f)),
                Some(_) => return Err(SchemeError::RuntimeError("internal error: unexpected state".into())),
                None => None
            };
            Ok(data)
        }
    }
}

fn prepare_arguments(arguments: &[AST], env: Rc<RefCell<Env>>) -> Result<Vec<DataType>, SchemeError> {
    let args_result: Result<Vec<_>, _> = arguments.iter()
        .map(|x| eval(Some(x.clone()), env.clone()))
        .collect();
    debug!("args: {:?}", args_result);
    if let Result::Err(ref e) = args_result { return Err(e.clone()); }

    let args = args_result.unwrap().iter()
        .filter(|x| x.is_some())
        .flat_map(|ref mut x| x.clone())
        .collect::<Vec<DataType>>();
    Ok(args)
}

fn execute(f: &Function, arguments: &[AST], env: Rc<RefCell<Env>>) -> Result<Option<DataType>, SchemeError> {
    match prepare_arguments(arguments, env.clone()) {
        Ok(args) => {
            f.call(args, env.clone()).and_then(|r| {
                match r {
                    Some(data) => Ok(Some(data)),
                    None => Ok(None)
                }
            })
        }
        Err(e) => return Err(e)
    }
}

pub fn datatype2str(value: &DataType) -> String {
    match value {
        &DataType::Bool(b) => format!("{}", b),
        &DataType::Pair(ref p) => format!("({:?} . {:?})", p.0, p.1),
        &DataType::Number(f) => format!("{}", f),
        &DataType::Symbol(ref s) => format!("'{}", s),
        &DataType::String(ref s) => format!("\"{}\"", s),
        &DataType::Proc(ref p) => format!("{:?}", p),
        &DataType::Lambda(ref p) => format!("{:?}", p),
        &DataType::List(ref v) => format!("'({})", v.iter()
            .map(|d| datatype2str(d)).collect::<Vec<_>>().join(" "))
    }
}

fn ast2datatype(value: &AST) -> Result<DataType, SchemeError> {
    match value {
        &AST::Children(ref v) => {
            let children_result: Result<Vec<_>, _> = v.iter().map(|ast| ast2datatype(&ast)).collect();
            if let Result::Err(ref e) = children_result { return Err(e.clone()); }

            let children = children_result.unwrap().into_iter().collect::<Vec<DataType>>();
            Ok(DataType::List(children))
        }
        &AST::Symbol(ref s) => {
            if s.starts_with("#") {
                if s.len() != 2 {
                    return Err("syntax error".into());
                }
                let c_option = s.chars().nth(1);
                if let Some('t') = c_option {
                    Ok(DataType::Bool(true))
                } else if let Some('f') = c_option {
                    Ok(DataType::Bool(false))
                } else {
                    Err("syntax error".into())
                }
            } else if s.starts_with("\"") && s.ends_with("\"") {
                Ok(DataType::String((&s[1..s.len() - 1]).to_string()))
            } else {
                Ok(DataType::Symbol(s.clone()))
            }
        }
        &AST::Integer(i) => Ok(DataType::Number(i as f64)),
        &AST::Float(f) => Ok(DataType::Number(f))
    }
}
