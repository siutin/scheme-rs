use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use log::debug;

use crate::types::{AST, Procedure, Function, DataType};
use crate::env::{Env, EnvRef, Environment};
use crate::SchemeError;

pub fn eval(mut ast_option: Option<AST>, mut env: EnvRef) -> Result<Option<DataType>, SchemeError> {
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
                    "quasiquote" => {
                        // (quasiquote template) — evaluate unquote/unquote-splicing inside
                        if let Some(template) = list.get(1) {
                            return Ok(Some(eval_quasiquote(template, env.clone())?));
                        } else {
                            return Err("quasiquote requires a template".into());
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
                        // Function shorthand: (define (f args...) body...)
                        // → (define f (lambda (args...) body...))
                        if let Some(&AST::Children(ref sig)) = s1 {
                            if let Some(&AST::Symbol(ref fname)) = sig.get(0) {
                                // Build lambda AST: (lambda (params...) body...)
                                let mut lambda_ast = vec![AST::Symbol("lambda".to_string())];
                                // Params are sig[1..] wrapped in a Children
                                let params = AST::Children(sig[1..].to_vec());
                                lambda_ast.push(params);
                                // Body is list[2..]
                                for expr in &list[2..] {
                                    lambda_ast.push(expr.clone());
                                }
                                let lambda_expr = AST::Children(lambda_ast);
                                // Evaluate the lambda
                                let data_option = eval(Some(lambda_expr), env.clone());
                                if let Ok(Some(DataType::Lambda(ref p))) = data_option {
                                    env.borrow().define(fname.clone(), DataType::Lambda(p.clone()));
                                } else if let Err(e) = data_option {
                                    return Err(e);
                                }
                                return Ok(None);
                            }
                        }
                        if let (Some(&AST::Symbol(ref s1)), Some(&ref a2)) = (s1, s2) {
                            match a2.clone() {
                                AST::Integer(i) => {
                                    env.borrow().define(s1.clone(), DataType::Integer(i));
                                }
                                AST::Float(f) => {
                                    env.borrow().define(s1.clone(), DataType::Float(f));
                                }
                                AST::Symbol(ref s) => {
                                    if s.len() > 1 && s.starts_with("#") {
                                        let c_option = s.chars().nth(1);
                                        if let Some('t') = c_option {
                                            env.borrow().define(s1.clone(), DataType::Bool(true));
                                        } else if let Some('f') = c_option {
                                            env.borrow().define(s1.clone(), DataType::Bool(false));
                                        } else {
                                            return Err("syntax error".into());
                                        }
                                    } else if s.starts_with("\"") && s.ends_with("\"") {
                                        env.borrow().define(s1.clone(), DataType::String((&s[1..s.len() - 1]).to_string()));
                                    } else {
                                        let data_option = env.borrow().get(s);
                                        if let Some(data) = data_option {
                                            env.borrow().define(s1.clone(), data);
                                        } else {
                                            return Err("symbol is not defined".into());
                                        }
                                    }
                                }
                                AST::Children(ref v) => {
                                    debug!("children: {:?}", v);

                                    let data_option = eval(Some(a2.clone()), env.clone());
                                    if let Ok(Some(DataType::Lambda(ref p))) = data_option {
                                        env.borrow().define(s1.clone(), DataType::Lambda(p.clone()));
                                    } else if let Ok(Some(DataType::List(ref v))) = data_option {
                                        env.borrow().define(s1.clone(), DataType::List(v.clone()));
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

                            let procedure_env = Env::new(HashMap::new(), Some(env.clone()));

                            let procedure = Procedure {
                                body: Rc::new(body_ast),
                                params: args_meta,
                                env: Rc::new(RefCell::new(procedure_env)) as EnvRef
                            };

                            return Ok(Some(DataType::Lambda(procedure)));
                        } else {
                            return Err("syntax error".into());
                        }
                    }
                    "let" => {
                        // Named let: (let name ((var init) ...) body...)
                        if let Some(&AST::Symbol(ref name)) = s1 {
                            // Second element must be bindings
                            if let Some(&AST::Children(ref bindings)) = s2 {
                                let body_exprs = &list[3..];
                                if body_exprs.is_empty() {
                                    return Err("named let requires a body".into());
                                }
                                // Extract param names and initial values
                                let mut params: Vec<DataType> = Vec::new();
                                let mut init_vals: Vec<DataType> = Vec::new();
                                for binding in bindings.iter() {
                                    if let AST::Children(ref pair) = binding {
                                        if let (Some(&AST::Symbol(ref pname)), Some(init)) = (pair.get(0), pair.get(1)) {
                                            params.push(DataType::Symbol(pname.clone()));
                                            match eval(Some(init.clone()), env.clone())? {
                                                Some(val) => init_vals.push(val),
                                                None => init_vals.push(DataType::Bool(false)),
                                            }
                                        } else {
                                            return Err("named let binding must be (name init)".into());
                                        }
                                    } else {
                                        return Err("named let bindings must be (name init) pairs".into());
                                    }
                                }
                                // Build body as implicit begin
                                let body_ast = if body_exprs.len() == 1 {
                                    body_exprs[0].clone()
                                } else {
                                    AST::Children(
                                        std::iter::once(AST::Symbol("begin".to_string()))
                                            .chain(body_exprs.iter().cloned())
                                            .collect()
                                    )
                                };
                                // Create the recursive procedure
                                // The proc env must contain a binding for `name` pointing to itself
                                let proc_env = Env::new(HashMap::new(), Some(env.clone()));
                                let proc_env_rc: EnvRef = Rc::new(RefCell::new(proc_env));
                                let procedure = Procedure {
                                    body: Rc::new(body_ast),
                                    params,
                                    env: proc_env_rc.clone(),
                                };
                                // Bind name to procedure in its own env (for recursion)
                                proc_env_rc.borrow().define(name.clone(), DataType::Lambda(procedure.clone()));
                                // Now call the procedure with init_vals
                                let call_env = Env::new(HashMap::new(), Some(proc_env_rc.clone()));
                                for (param, val) in procedure.params.iter().zip(init_vals.into_iter()) {
                                    if let DataType::Symbol(ref pname) = param {
                                        call_env.define(pname.clone(), val);
                                    }
                                }
                                let call_env_rc: EnvRef = Rc::new(RefCell::new(call_env));
                                // TAIL: evaluate the body in the call env
                                ast_option = Some((*procedure.body).clone());
                                env = call_env_rc;
                                continue;
                            } else {
                                return Err("named let requires bindings".into());
                            }
                        }
                        // Regular let: (let ((var init) ...) body...)
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
                            let let_env = Env::new(local, Some(env.clone()));
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
                            env = Rc::new(RefCell::new(let_env)) as EnvRef;
                            continue;
                        } else {
                            return Err("let requires bindings and a body".into());
                        }
                    }
                    "let*" => {
                        // (let* ((var init) ...) body...)
                        // Sequential bindings: each init sees previous bindings
                        if let (Some(&AST::Children(ref bindings)), Some(_)) = (s1, s2) {
                            let body_exprs = &list[2..];
                            if body_exprs.is_empty() {
                                return Err("let* requires a body".into());
                            }
                            // Build env chain: each binding gets its own env layer
                            let mut current_env = env.clone();
                            for binding in bindings.iter() {
                                if let AST::Children(ref pair) = binding {
                                    if let (Some(&AST::Symbol(ref name)), Some(init)) = (pair.get(0), pair.get(1)) {
                                        match eval(Some(init.clone()), current_env.clone()) {
                                            Ok(Some(val)) => {
                                                let mut local = HashMap::new();
                                                local.insert(name.clone(), val);
                                                let new_env = Env::new(local, Some(current_env));
                                                current_env = Rc::new(RefCell::new(new_env)) as EnvRef;
                                            }
                                            Ok(None) => {
                                                let mut local = HashMap::new();
                                                local.insert(name.clone(), DataType::Bool(false));
                                                let new_env = Env::new(local, Some(current_env));
                                                current_env = Rc::new(RefCell::new(new_env)) as EnvRef;
                                            }
                                            Err(e) => return Err(e),
                                        }
                                    } else {
                                        return Err("let* binding must be (name init)".into());
                                    }
                                } else {
                                    return Err("let* bindings must be (name init) pairs".into());
                                }
                            }
                            // Build body as implicit begin
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
                            env = current_env;
                            continue;
                        } else {
                            return Err("let* requires bindings and a body".into());
                        }
                    }
                    "letrec" => {
                        // (letrec ((var init) ...) body...)
                        // Recursive bindings: all vars visible to all inits
                        if let (Some(&AST::Children(ref bindings)), Some(_)) = (s1, s2) {
                            let body_exprs = &list[2..];
                            if body_exprs.is_empty() {
                                return Err("letrec requires a body".into());
                            }
                            // Create env with all names pre-bound to unspecified
                            let mut local = HashMap::new();
                            for binding in bindings.iter() {
                                if let AST::Children(ref pair) = binding {
                                    if let Some(&AST::Symbol(ref name)) = pair.get(0) {
                                        local.insert(name.clone(), DataType::Bool(false));
                                    }
                                }
                            }
                            let letrec_env = Env::new(local, Some(env.clone()));
                            let letrec_env_rc: EnvRef = Rc::new(RefCell::new(letrec_env));
                            // Now evaluate each init in this env and update bindings
                            for binding in bindings.iter() {
                                if let AST::Children(ref pair) = binding {
                                    if let (Some(&AST::Symbol(ref name)), Some(init)) = (pair.get(0), pair.get(1)) {
                                        match eval(Some(init.clone()), letrec_env_rc.clone()) {
                                            Ok(Some(val)) => {
                                                letrec_env_rc.borrow().define(name.clone(), val);
                                            }
                                            Ok(None) => {}
                                            Err(e) => return Err(e),
                                        }
                                    }
                                }
                            }
                            // Build body as implicit begin
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
                            env = letrec_env_rc;
                            continue;
                        } else {
                            return Err("letrec requires bindings and a body".into());
                        }
                    }
                    "and" => {
                        // (and expr...) — short-circuit, returns last truthy or first #f
                        if list.len() == 1 {
                            return Ok(Some(DataType::Bool(true)));
                        }
                        let mut result = DataType::Bool(true);
                        for i in 1..list.len() {
                            let is_last = i == list.len() - 1;
                            match eval(Some(list[i].clone()), env.clone())? {
                                Some(DataType::Bool(false)) => return Ok(Some(DataType::Bool(false))),
                                Some(val) => {
                                    if is_last {
                                        result = val;
                                    }
                                    // Continue evaluating
                                }
                                None => {
                                    if is_last {
                                        return Ok(None);
                                    }
                                }
                            }
                        }
                        return Ok(Some(result));
                    }
                    "or" => {
                        // (or expr...) — short-circuit, returns first truthy or last #f
                        if list.len() == 1 {
                            return Ok(Some(DataType::Bool(false)));
                        }
                        for i in 1..list.len() - 1 {
                            match eval(Some(list[i].clone()), env.clone())? {
                                Some(DataType::Bool(false)) => {} // continue
                                Some(val) => return Ok(Some(val)),
                                None => {} // continue
                            }
                        }
                        // TAIL: last expression
                        ast_option = Some(list[list.len() - 1].clone());
                        continue;
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
                    "do" => {
                        // (do ((var init step) ...) (test result ...) body ...)
                        let bindings = list.get(1);
                        let test_clause = list.get(2);
                        let body_exprs = &list[3..];

                        // Parse bindings: (var init step?) pairs
                        let mut var_names: Vec<String> = Vec::new();
                        let mut var_vals: Vec<DataType> = Vec::new();
                        let mut var_steps: Vec<Option<AST>> = Vec::new();

                        if let Some(&AST::Children(ref binds)) = bindings {
                            for binding in binds.iter() {
                                if let AST::Children(ref parts) = binding {
                                    if let Some(&AST::Symbol(ref name)) = parts.get(0) {
                                        var_names.push(name.clone());
                                        // init
                                        if let Some(init) = parts.get(1) {
                                            match eval(Some(init.clone()), env.clone())? {
                                                Some(val) => var_vals.push(val),
                                                None => var_vals.push(DataType::Bool(false)),
                                            }
                                        } else {
                                            var_vals.push(DataType::Bool(false));
                                        }
                                        // step (optional)
                                        var_steps.push(parts.get(2).cloned());
                                    } else {
                                        return Err("do binding must start with a symbol".into());
                                    }
                                } else {
                                    return Err("do bindings must be (var init step?)".into());
                                }
                            }
                        } else {
                            return Err("do requires bindings".into());
                        }

                        // Parse test clause: (test result ...)
                        let (test_ast, result_exprs) = if let Some(&AST::Children(ref tc)) = test_clause {
                            let test = tc.get(0).cloned();
                            let results = tc[1..].to_vec();
                            (test, results)
                        } else {
                            return Err("do requires a test clause".into());
                        };

                        // Main loop
                        loop {
                            // Create do env with current var values
                            let mut do_local = HashMap::new();
                            for (name, val) in var_names.iter().zip(var_vals.iter()) {
                                do_local.insert(name.clone(), val.clone());
                            }
                            let do_env = Env::new(do_local, Some(env.clone()));
                            let do_env_rc: EnvRef = Rc::new(RefCell::new(do_env));

                            // Evaluate test
                            let test_result = match test_ast {
                                Some(ref t) => eval(Some(t.clone()), do_env_rc.clone())?,
                                None => Some(DataType::Bool(false)),
                            };

                            if let Some(DataType::Bool(true)) = test_result {
                                // Test passed — evaluate result expressions
                                if result_exprs.is_empty() {
                                    return Ok(None);
                                }
                                // Evaluate all but last for side effects, return last
                                for expr in &result_exprs[..result_exprs.len() - 1] {
                                    eval(Some(expr.clone()), do_env_rc.clone())?;
                                }
                                // TAIL: last result expression
                                ast_option = Some(result_exprs[result_exprs.len() - 1].clone());
                                env = do_env_rc;
                                // Break out of do loop, continue eval loop
                                break;
                            }

                            // Evaluate body expressions for side effects
                            for expr in body_exprs {
                                eval(Some(expr.clone()), do_env_rc.clone())?;
                            }

                            // Compute step values
                            let mut new_vals: Vec<DataType> = Vec::new();
                            for (i, step) in var_steps.iter().enumerate() {
                                if let Some(ref step_ast) = step {
                                    match eval(Some(step_ast.clone()), do_env_rc.clone())? {
                                        Some(val) => new_vals.push(val),
                                        None => new_vals.push(var_vals[i].clone()),
                                    }
                                } else {
                                    // No step — keep current value
                                    new_vals.push(var_vals[i].clone());
                                }
                            }
                            var_vals = new_vals;
                        }
                        continue;
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
                                        let proc_env = Env::new(HashMap::new(), Some(proc_env_ref));

                                        for (name_ref, value_ref) in params.iter().zip(args.into_iter()) {
                                            if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                                proc_env.define(name.to_string(), value.clone());
                                            } else {
                                                return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                                            }
                                        }

                                        // TAIL: reassign and continue
                                        ast_option = Some((*body).clone());
                                        env = Rc::new(RefCell::new(proc_env)) as EnvRef;
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
                                            let proc_env = Env::new(HashMap::new(), Some(p.env.clone()));
                                            for (name_ref, value_ref) in p.params.iter().zip(args.into_iter()) {
                                                if let (Some(&DataType::Symbol(ref name)), Some(ref value)) = (Some(name_ref), Some(value_ref)) {
                                                    proc_env.define(name.to_string(), value.clone());
                                                } else {
                                                    return Err(SchemeError::RuntimeError("internal error: unexpected state".into()))
                                                }
                                            }
                                            proc_env
                                        }
                                        Err(e) => return Err(e)
                                    }
                                }
                                None => {
                                    Env::new(HashMap::new(), Some(p.env.clone()))
                                }
                            };
                            // TAIL: reassign and continue
                            ast_option = Some((*p.body).clone());
                            env = Rc::new(RefCell::new(proc_env)) as EnvRef;
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

fn prepare_arguments(arguments: &[AST], env: EnvRef) -> Result<Vec<DataType>, SchemeError> {
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

fn execute(f: &Function, arguments: &[AST], env: EnvRef) -> Result<Option<DataType>, SchemeError> {
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

/// Evaluate a quasiquote template — recursively walk the AST, evaluating
/// unquote forms and splicing unquote-splicing forms into lists.
fn eval_quasiquote(template: &AST, env: EnvRef) -> Result<DataType, SchemeError> {
    match template {
        // (unquote expr) — evaluate expr
        &AST::Children(ref v) if v.len() == 2 => {
            if let Some(&AST::Symbol(ref s)) = v.get(0) {
                if s == "unquote" {
                    match eval(Some(v[1].clone()), env)? {
                        Some(d) => return Ok(d),
                        None => return Ok(DataType::Bool(false)),
                    }
                }
            }
            // Not unquote — process each element recursively
            let mut result: Vec<DataType> = Vec::new();
            for elem in v.iter() {
                // Check for unquote-splicing
                if let AST::Children(ref inner) = elem {
                    if inner.len() == 2 {
                        if let Some(&AST::Symbol(ref s)) = inner.get(0) {
                            if s == "unquote-splicing" {
                                match eval(Some(inner[1].clone()), env.clone())? {
                                    Some(DataType::List(splice)) => {
                                        result.extend(splice);
                                        continue;
                                    }
                                    Some(_) => return Err("unquote-splicing requires a list".into()),
                                    None => continue,
                                }
                            }
                        }
                    }
                }
                result.push(eval_quasiquote(elem, env.clone())?);
            }
            Ok(DataType::List(result))
        }
        // Children with len != 2 — process each element recursively
        &AST::Children(ref v) => {
            let mut result: Vec<DataType> = Vec::new();
            for elem in v.iter() {
                if let AST::Children(ref inner) = elem {
                    if inner.len() == 2 {
                        if let Some(&AST::Symbol(ref s)) = inner.get(0) {
                            if s == "unquote-splicing" {
                                match eval(Some(inner[1].clone()), env.clone())? {
                                    Some(DataType::List(splice)) => {
                                        result.extend(splice);
                                        continue;
                                    }
                                    Some(_) => return Err("unquote-splicing requires a list".into()),
                                    None => continue,
                                }
                            }
                        }
                    }
                }
                result.push(eval_quasiquote(elem, env.clone())?);
            }
            Ok(DataType::List(result))
        }
        // Atoms — convert directly
        &AST::Integer(i) => Ok(DataType::Integer(i)),
        &AST::Float(f) => Ok(DataType::Float(f)),
        &AST::Symbol(ref s) => Ok(DataType::Symbol(s.clone())),
    }
}
