use std::collections::HashMap;
use std::cell::{RefCell, RefMut};
use std::borrow::BorrowMut;
use std::sync::{Arc};

#[derive(Clone, Debug)]
enum AST {
	Fixnum(u32),
	Float(f64),
	Symbol(String),
	Children(Vec<AST>)
}

#[derive(Debug)]
struct ReadFromTokenResult {
	remain: Vec<String>,
	result: AST
}

#[derive(Debug)]
enum Varible {
	Fixnum(u32),
	Float(f64),
	Symbol(String)
}

fn main() {
	// println!("Hello, world!");
	let program = "(begin (define r 10) (* pi (* r r)))";
	println!("program: {}", program);
	let tokens = tokenize(program);
	println!("tokens: {:?}", tokens);
	let ast = read_from_tokens(tokens.clone());
	println!("ast: {:?}", ast);
	if ast.is_ok() {
		let env = RefCell::new(HashMap::new());
		let p = eval(ast.unwrap().result, &env);
		match p {
			Ok(r) => println!("p: {:?}", r),
			Err(e) => panic!("ERROR: {}", e)
		}
	}

}

fn tokenize(program: &str) -> Vec<String>
{
	let iterator = program.chars();
	let iterator1 = iterator.clone();
	let mut iterator2 = iterator1.clone();

	let count = iterator1.count();
	let mut vec:Vec<char> = Vec::with_capacity(count);

	// println!("{:?}", iterator2);

	loop {
		match iterator2.next() {
			Some(x) => {
				if x == '(' {
					vec.push(' ');
					vec.push('(');
					vec.push(' ');
				} else if x == ')' {
					vec.push(' ');
					vec.push(')');
					vec.push(' ');
				} else {
					vec.push(x);
				}
			},
			None => { break; }
		}
	}

	// println!("vec count: {}", (&mut vec).len());
	// println!("{:?}", vec);

	let s:String = vec.into_iter().collect();
	let ss:Vec<String> = s.split_whitespace().map(|x| x.to_string() ).collect();
	ss
}

fn read_from_tokens(mut tokens:Vec<String>) -> Result<ReadFromTokenResult, &'static str> {
	if tokens.len() > 0 {
		let mut token = tokens.remove(0);

		if token == "(" {
			let mut vec:Vec<AST> = vec![];
			let mut tmp_tokens = tokens.clone();

			while tmp_tokens[0] != ")" {
				match read_from_tokens(tmp_tokens.clone()) {
					Ok(mut data) => {
						vec.push(data.result);
						tmp_tokens = data.remain.clone();
					},
					Err(e) => { return Err(e) }
				}
			}
			tmp_tokens.remove(0);
			Ok(
				ReadFromTokenResult {
					remain: tmp_tokens,
					result: AST::Children(vec)
				}
			)
		} else if token == ")" {
			Err("unexpected )")
		} else {
			Ok(
				ReadFromTokenResult {
					remain: tokens,
					result: atom(&token)
				}
			)
		}
	} else {
		Err("unexpected EOF while reading")
	}
}

fn atom(token: &str) -> AST {
	let to_int = token.parse::<u32>();
	let to_float = token.parse::<f64>();

	if to_int.is_ok() {
		AST::Fixnum(to_int.unwrap_or_default())
	} else if to_float.is_ok() {
		AST::Float(to_float.unwrap_or_default())
	} else {
		AST::Symbol(token.to_string())
	}
}

fn eval(ast: AST, mut env: &RefCell<HashMap<String, Varible>>) -> Result<AST, &'static str> {
	if let AST::Children(list) = ast {
			let solved_list = {
				let has_children = list.iter().any(|x| if let AST::Children(ref l) = *x { true } else { false });
				if has_children {
					list.into_iter().map(|x| {
						return eval(x, &env).unwrap();
					}).collect::<Vec<AST>>()
				} else {
					list
				}
			};

		if let AST::Symbol(ref s0) = solved_list[0] {
			if s0 == "define" {
				if let AST::Symbol(ref s1) = solved_list[1].clone() {
					match solved_list[2] {
						AST::Fixnum(i) => env.borrow_mut().insert(s1.clone(), Varible::Fixnum(i)),
						AST::Float(f) => env.borrow_mut().insert(s1.clone(), Varible::Float(f)),
						AST::Symbol(ref s) => env.borrow_mut().insert(s1.clone(), Varible::Symbol(s.clone())),
						AST::Children(ref l) => { return Err("should not reach here"); }
					};
				} else {
					return Err("definition name must be a symbol");
				}
			}
		}

		Ok(solved_list[0].clone())
	} else {
		Ok(ast)
	}
}
