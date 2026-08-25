use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use log::debug;

use crate::types::{AST, Procedure, Function, DataType};
use crate::env::Env;
use crate::SchemeError;

pub fn eval(mut ast_option: Option<AST>, mut env: Rc<RefCell<Env>>) -> Result<Option<DataType>, SchemeError> {
    loop {
    debug!("eval");
    debug!("{:?}", ast_option);
    match ast_option.take() {
        Some(AST::Symbol(s)) => {
            debug!("ast is a symbol: {:?}", s);
            if s.starts_with("#") {
                if s.len() != 2 {
                    return Err("syntax error".into());
                }
                let c_option = s.chars().nth(1);
                if let Some('t') = c_option {
                    return Ok(Some(DataType::Bool(true)));
                } else if let Some('f') = c_option {
                    return Ok(Some(DataType::Bool(false)));
                } else {
                    return Err("syntax error".into());
                }
            } else if s.len() > 1 && s.starts_with("'") {
                let slice = &s[1..s.len()];
                return Ok(Some(DataType::Symbol(slice.to_string())));
            } else if s.starts_with("\"") && s.ends_with("\"") {
                return Ok(Some(DataType::String((&s[1..s.len() - 1]).to_string())));
            } else {
                match env.borrow().get(&s) {
                    Some(data) => return Ok(Some(data)),
                    None => return Err("symbol is not defined.".into())
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
                                    Ok(data) => return Ok(Some(data)),
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
                                        // TAIL: reassign and continue instead of recursive eval
                                        true => { ast_option = Some(conseq.clone()); continue; }
                                        false => { ast_option = Some(alt.clone()); continue; }
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
                                    env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Integer(i));
                                }
                                AST::Float(f) => {
                                    let env_borrow_mut = env.borrow_mut();
                                    env_borrow_mut.local.borrow_mut().insert(s1.clone(), DataType::Float(f));
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
                        if let (Some(&AST::Children(ref args)), Some(_)) = (s1, s2) {
                            // Body is all expressions after args: list[2..]
                            let body_exprs = &list[2..];
                            if body_exprs.is_empty() {
                                return Err("lambda requires a body".into());
                            }

                            // Convert multi-expression body into implicit begin
                            let body_ast = if body_exprs.len() == 1 {
                                body_exprs[0].clone()
                            } else {
                                AST::Children(
                                    std::iter::once(AST::Symbol("begin".to_string()))
                                        .chain(body_exprs.iter().cloned())
                                        .collect()
                                )
                            };

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

                            let procedure = Procedure {
                                body: Rc::new(body_ast),
                                params: args_meta,
                                env: Rc::new(RefCell::new(procedure_env))
                            };

                            return Ok(Some(DataType::Lambda(procedure)));
                        } else {
                            return Err("syntax error".into());
                        }
                    }
                    "let" => {
                        // (let ((var init) ...) body...)
                        if let (Some(&AST::Children(ref bindings)), Some(_)) = (s1, s2) {
                            let body_exprs = &list[2..];
                            if body_exprs.is_empty() {
                                return Err("let requires a body".into());
                            }
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
                            // Convert multi-expression body into implicit begin
                            let body_ast = if body_exprs.len() == 1 {
                                body_exprs[0].clone()
                            } else {
                                AST::Children(
                                    std::iter::once(AST::Symbol("begin".to_string()))
                                        .chain(body_exprs.iter().cloned())
                                        .collect()
                                )
                            };
                            // TAIL: reassign and continue
                            ast_option = Some(body_ast);
                            env = Rc::new(RefCell::new(let_env));
                            continue;
                        } else {
                            return Err("let requires bindings and a body".into());
                        }
                    }
                    "cond" => {
                        // (cond (test expr...) ... (else expr...))
                        let mut tail_expr: Option<AST> = None;
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
                                        tail_expr = Some(body_ast.clone());
                                    }
                                    break;
                                }
                                if let Some(test_ast) = test {
                                    match eval(Some(test_ast.clone()), env.clone()) {
                                        Ok(Some(DataType::Bool(true))) => {
                                            if let Some(body_ast) = body {
                                                tail_expr = Some(body_ast.clone());
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
                        // TAIL: if we found a matching clause, continue with its body
                        match tail_expr {
                            Some(expr) => { ast_option = Some(expr); continue; }
                            None => return Ok(Some(DataType::Bool(false))),
                        }
                    }
                    "set!" => {
                        // (set! var expr) — mutate existing binding
                        if let (Some(&AST::Symbol(ref name)), Some(val_ast)) = (s1, s2) {
                            match eval(Some(val_ast.clone()), env.clone()) {
                                Ok(Some(val)) => {
                                    if env.borrow().set(name, val) {
                                        return Ok(None);
                                    } else {
                                        return Err(SchemeError::UndefinedSymbol(name.clone()));
                                    }
                                }
                                Ok(None) => {
                                    if env.borrow().set(name, DataType::Bool(false)) {
                                        return Ok(None);
                                    } else {
                                        return Err(SchemeError::UndefinedSymbol(name.clone()));
                                    }
                                }
                                Err(e) => return Err(e),
                            }
                        } else {
                            return Err("set! requires a symbol and a value".into());
                        }
                    }
                    "when" => {
                        // (when test body...) — eval body if test is true
                        if let Some(test_ast) = s1 {
                            let body_exprs = &list[2..];
                            if body_exprs.is_empty() {
                                return Err("when requires a body".into());
                            }
                            match eval(Some(test_ast.clone()), env.clone()) {
                                Ok(Some(DataType::Bool(true))) => {
                                    // Convert multi-expression body into implicit begin
                                    let body_ast = if body_exprs.len() == 1 {
                                        body_exprs[0].clone()
                                    } else {
                                        AST::Children(
                                            std::iter::once(AST::Symbol("begin".to_string()))
                                                .chain(body_exprs.iter().cloned())
                                                .collect()
                                        )
                                    };
                                    // TAIL
                                    ast_option = Some(body_ast);
                                    continue;
                                }
                                Ok(Some(DataType::Bool(false))) | Ok(None) => return Ok(None),
                                Ok(_) => return Err("when requires a boolean test".into()),
                                Err(e) => return Err(e),
                            }
                        } else {
                            return Err("when requires a test and a body".into());
                        }
                    }
                    "unless" => {
                        // (unless test body...) — eval body if test is false
                        if let Some(test_ast) = s1 {
                            let body_exprs = &list[2..];
                            if body_exprs.is_empty() {
                                return Err("unless requires a body".into());
                            }
                            match eval(Some(test_ast.clone()), env.clone()) {
                                Ok(Some(DataType::Bool(false))) => {
                                    // Convert multi-expression body into implicit begin
                                    let body_ast = if body_exprs.len() == 1 {
                                        body_exprs[0].clone()
                                    } else {
                                        AST::Children(
                                            std::iter::once(AST::Symbol("begin".to_string()))
                                                .chain(body_exprs.iter().cloned())
                                                .collect()
                                        )
                                    };
                                    // TAIL
                                    ast_option = Some(body_ast);
                                    continue;
                                }
                                Ok(Some(DataType::Bool(true))) | Ok(None) => return Ok(None),
                                Ok(_) => return Err("unless requires a boolean test".into()),
                                Err(e) => return Err(e),
                            }
                        } else {
                            return Err("unless requires a test and a body".into());
                        }
                    }
                    "case" => {
                        // (case key clause...) — each clause is ((vals...) body) or (else body)
                        if let Some(key_ast) = s1 {
                            match eval(Some(key_ast.clone()), env.clone()) {
                                Ok(Some(key_val)) => {
                                    let mut tail_expr: Option<AST> = None;
                                    for i in 2..list.len() {
                                        if let Some(&AST::Children(ref clause)) = list.get(i) {
                                            let vals = clause.get(0);
                                            let body = clause.get(1);
                                            // Check for else
                                            let is_else = match vals {
                                                Some(&AST::Symbol(ref s)) if s == "else" => true,
                                                _ => false,
                                            };
                                            if is_else {
                                                if let Some(body_ast) = body {
                                                    tail_expr = Some(body_ast.clone());
                                                }
                                                break;
                                            }
                                            // Check if key matches any value in the list
                                            if let Some(&AST::Children(ref val_list)) = vals {
                                                let matched = val_list.iter().any(|v| {
                                                    if let Ok(d) = ast2datatype(v) {
                                                        d == key_val
                                                    } else {
                                                        false
                                                    }
                                                });
                                                if matched {
                                                    if let Some(body_ast) = body {
                                                        tail_expr = Some(body_ast.clone());
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    // TAIL
                                    match tail_expr {
                                        Some(expr) => { ast_option = Some(expr); continue; }
                                        None => return Ok(Some(DataType::Bool(false))),
                                    }
                                }
                                Ok(None) => return Ok(None),
                                Err(e) => return Err(e),
                            }
                        } else {
                            return Err("case requires a key".into());
                        }
                    }
                    _ => {
                        debug!("Some(AST::Symbol) but not define");
                        debug!("proc_key : {}", s0);

                        // get() returns owned DataType, no extra clone needed
                        let lookup = env.borrow().get(s0);
                        match lookup {
                            Some(DataType::Proc(ref f)) => {
                                let slice = &list[1..list.len()];
                                return execute(f, slice, env);
                            }
                            Some(DataType::Lambda(p)) => {
                                // Move out of p — body is Rc<AST>, cheap clone
                                let slice = &list[1..list.len()];
                                let body = p.body.clone();
                                let params = p.params;
                                let proc_env_ref = p.env;
                                match prepare_arguments(slice, env.clone()) {
                                    Ok(args) => {
                                        let procedure_local = Box::new(RefCell::new(HashMap::new()));

                                        for (name_ref, value_ref) in params.iter().zip(args.into_iter()) {
                                            if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                                procedure_local.borrow_mut().insert(name.to_string(), value.clone());
                                            } else {
                                                return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                                            }
                                        }

                                        let proc_env = Env {
                                            local: procedure_local,
                                            parent: Some(Box::new(proc_env_ref))
                                        };

                                        // TAIL: reassign and continue
                                        ast_option = Some((*body).clone());
                                        env = Rc::new(RefCell::new(proc_env));
                                        continue;
                                    }
                                    Err(e) => return Err(e)
                                }
                            }
                            Some(_) | None => return Err("symbol is not defined.".into())
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
                            return match rest_option {
                                Some(rest) => execute(f, rest, env),
                                None => execute(f, &vec![], env)
                            };
                        }
                        Ok(Some(DataType::Lambda(p))) => {
                            let proc_env = match rest_option {
                                Some(rest) => {
                                    match prepare_arguments(rest, env.clone()) {
                                        Ok(args) => {
                                            let procedure_local = Box::new(RefCell::new(HashMap::new()));
                                            for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
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
                            // TAIL: reassign and continue
                            ast_option = Some((*p.body).clone());
                            env = Rc::new(RefCell::new(proc_env));
                            continue;
                        }
                        Ok(_) => { return Err("unsupported data type on first element".into()); }
                        Err(e) => { return Err(e); }
                    }
                } else {
                    return Err("syntax error".into());
                }
            }
        }
        Some(AST::Integer(i)) => return Ok(Some(DataType::Integer(i))),
        Some(AST::Float(f)) => return Ok(Some(DataType::Float(f))),
        None => return Ok(None),
    }
    } // end loop
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
        &DataType::Integer(i) => format!("{}", i),
        &DataType::Float(f) => format!("{}", f),
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
        &AST::Integer(i) => Ok(DataType::Integer(i)),
        &AST::Float(f) => Ok(DataType::Float(f))
    }
}
