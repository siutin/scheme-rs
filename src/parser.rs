use log::debug;

use crate::types::{AST, ReadFromTokenResult};
use crate::SchemeError;

pub fn parse(program: &str) -> Result<ReadFromTokenResult, SchemeError> {
    debug!("program: {}", program);
    let wrap_program = format!("(begin {})", program);

    let tokens = tokenize(&wrap_program);
    debug!("tokens: {:?}", tokens);
    let ast = read_from_tokens(tokens.clone());
    debug!("ast: {:?}", ast);
    return ast;
}

fn tokenize(program: &str) -> Vec<String>
{
    // Strip Scheme comments: ; to end of line (but not inside strings)
    let stripped: String = program.lines()
        .map(|line| {
            let mut in_string = false;
            let mut result = String::new();
            for c in line.chars() {
                if c == '"' {
                    in_string = !in_string;
                }
                if c == ';' && !in_string {
                    break;
                }
                result.push(c);
            }
            result
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Tokenize: split on whitespace and parens, but keep strings intact
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_string = false;

    for c in stripped.chars() {
        if in_string {
            current.push(c);
            if c == '"' {
                tokens.push(current.clone());
                current.clear();
                in_string = false;
            }
        } else if c == '"' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            current.push(c);
            in_string = true;
        } else if c == '(' || c == ')' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            tokens.push(c.to_string());
        } else if c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn read_from_tokens(mut tokens: Vec<String>) -> Result<ReadFromTokenResult, SchemeError> {
    if tokens.len() > 0 {
        let token = tokens.remove(0);

        if token == "(" {
            let mut vec: Vec<AST> = vec![];
            let mut tmp_tokens = tokens.clone();

            if tmp_tokens.is_empty() {
                return Err("syntax error".into());
            }

            while !tmp_tokens.is_empty() {
                if tmp_tokens.first().unwrap() == ")" {
                    break
                } else {
                    let start_quote_option = match tmp_tokens.clone().first() {
                        Some(first_word) => {
                            if first_word.starts_with('\"') {
                                debug!("detect a start quote of string");
                                Some(tmp_tokens.clone())
                            } else {
                                None
                            }
                        }
                        None => None
                    };
                    if let Some(rest_str) = start_quote_option {
                        debug!("rest_str: {:?}", rest_str);
                        match rest_str.iter().position(|string_tag| if string_tag.ends_with('\"') { true } else { false }) {
                            Some(i) => {
                                debug!("detect an end quote of string");
                                let str_result = (rest_str[0..i + 1]).join(" ");
                                let rest_tokens = (rest_str[i + 1..]).iter().map(|&ref x| x.to_string()).collect::<Vec<String>>();
                                debug!("str_result: {:?}", str_result);
                                debug!("rest_tokens: {:?}", rest_tokens);
                                vec.push(AST::Symbol(str_result));
                                tmp_tokens = rest_tokens.clone();
                            }
                            None => { return Err("can not find an end quote".into()); }
                        }
                    } else {
                        match read_from_tokens(tmp_tokens.clone()) {
                            Ok(data) => {
                                vec.push(data.result);
                                tmp_tokens = data.remain.clone();
                            }
                            Err(e) => { return Err(e); }
                        }
                    }
                }
            }
            if tmp_tokens.is_empty() {
                return Err("syntax error".into());
            }
            tmp_tokens.remove(0);
            Ok(
                ReadFromTokenResult {
                    remain: tmp_tokens,
                    result: AST::Children(vec)
                }
            )
        } else if token == ")" {
            Err("unexpected )".into())
        } else if token == "'" {
            // Quote shorthand: read the next form and wrap in (quote <form>)
            if tokens.is_empty() {
                return Err("unexpected EOF after quote".into());
            }
            match read_from_tokens(tokens) {
                Ok(data) => {
                    Ok(
                        ReadFromTokenResult {
                            remain: data.remain,
                            result: AST::Children(vec![AST::Symbol("quote".to_string()), data.result])
                        }
                    )
                }
                Err(e) => Err(e)
            }
        } else {
            Ok(
                ReadFromTokenResult {
                    remain: tokens,
                    result: atom(&token)
                }
            )
        }
    } else {
        Err("unexpected EOF while reading".into())
    }
}

fn atom(token: &str) -> AST {
    let to_int = token.parse::<i64>();
    let to_float = token.parse::<f64>();

    if to_int.is_ok() {
        AST::Integer(to_int.unwrap_or_default())
    } else if to_float.is_ok() {
        AST::Float(to_float.unwrap_or_default())
    } else {
        AST::Symbol(token.to_string())
    }
}
