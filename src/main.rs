use std::collections::HashMap;
use std::cell::RefCell;

#[derive(Clone, Debug)]
enum AST {
	Integer(u64),
	Float(f64),
	Symbol(String),
	Children(Vec<AST>)
}

#[derive(Debug)]
struct ReadFromTokenResult {
	remain: Vec<String>,
	result: AST
}

#[derive(Clone, Debug)]
enum DataType {
	Integer(u64),
	Float(f64),
	Symbol(String)
}

#[derive(Debug)]
struct Env<'a> {
	local: &'a RefCell<HashMap<String, DataType>>,
	functions:	&'a RefCell<HashMap<&'a str, Box<Fn(Vec<DataType>) -> Result<Option<DataType>, &'static str>>>>,
	parent: Option<Box<RefCell<Env<'a>>>>
}

impl <'a> Env<'a> {
	fn get(&mut self, key: &String) -> Option<DataType> {
		let local_borrow = self.local.borrow_mut();
		match local_borrow.get(key) {
			Some(&DataType::Integer(i)) => Some(DataType::Integer(i)),
			Some(&DataType::Float(f)) => Some(DataType::Float(f)),
			Some(&DataType::Symbol(ref ss)) =>Some(DataType::Symbol(ss.clone())),
			None => {
				match self.parent {
					Some(ref some_parent) => {
						let mut parent_borrow = some_parent.borrow_mut();
						parent_borrow.get(key)
					},
					None => None
				}
			}
		}
	}
}

fn main() {
	// println!("Hello, world!");

	// pre-defined commands experiment
	let mut func_hashmap:HashMap<&str, Box<Fn(Vec<DataType>) -> Result<Option<DataType>, &'static str>>> = HashMap::new();

	func_hashmap.insert("hello", Box::new(|vec|{
		println!("Function Args: {:?}", vec);
		println!("Hello, {:?}!", vec);
		Ok(None)
	}));

	func_hashmap.insert("add", Box::new(|vec|{
		println!("Function Args: {:?}", vec);
		let is_all_integers = vec.iter().all(|x| if let DataType::Integer(_) = *x { true } else { false }); // check it's not an integer list
		let is_all_integer_or_floats = vec.iter().all(|x|
			if let DataType::Integer(_) = *x { true } else if let DataType::Float(_) = *x { true } else { false }
		); // check it's not an float list
		if !is_all_integer_or_floats {
			return Err("wrong argument datatype");
		}

		let vec_boxed = Box::new(vec);
		let vec_boxed2 = vec_boxed.clone();

		let desc = vec_boxed.into_iter().map(|x|
			match x {
				DataType::Integer(i) => i.to_string(),
				DataType::Float(f) => f.to_string(),
				DataType::Symbol(_) => panic!("Something went wrong")
			}
 		).collect::<Vec<String>>().join(" + ");
		println!("Description: {}", desc);

		if is_all_integers {
			let result = vec_boxed2.into_iter().fold(0,|o,n|
				if let DataType::Integer(i) = n {
					o + i
				} else {
					panic!("Something went wrong")
				}
			);
			Ok(Some(DataType::Integer(result)))
		} else if is_all_integer_or_floats {
			let result = vec_boxed2.into_iter().fold(0.0,|o,n|
				if let DataType::Integer(i) = n {
					o + (i as f64)
				} else if let DataType::Float(f) = n {
					o + f
				} else {
					panic!("Something went wrong")
				}
			);
			Ok(Some(DataType::Float(result)))
		} else {
			Err("Something went wrong")
		}
	}));

	println!("func_hashmap start");
	for (i,key) in func_hashmap.keys().enumerate() {
		println!("{} => {}", i + 1, key);
		match func_hashmap.get(key) {
			Some(f) => {
				match f(vec![DataType::Integer(1), DataType::Integer(2), DataType::Float(5.1)]) {
					Ok(result) => { println!("Execution is good. Result: {:?}", result); }
					Err(_) => { println!("Execution is failed"); }
				}
			}
			None => {}
		}
	}
	println!("func_hashmap end");

	let program = "(begin (define r 10) (* pi (* r r)))";
	println!("program: {}", program);
	let tokens = tokenize(program);
	println!("tokens: {:?}", tokens);
	let ast = read_from_tokens(tokens.clone());
	println!("ast: {:?}", ast);
	if ast.is_ok() {
		let global = RefCell::new(HashMap::new());

		global.borrow_mut().insert("begin".to_string(), DataType::Integer(1));
		global.borrow_mut().insert("*".to_string(), DataType::Integer(2));
		global.borrow_mut().insert("pi".to_string(), DataType::Float(3.141592654));

		let functions = RefCell::new(func_hashmap);
		let env = RefCell::new(
			Env {
				local: &global,
				functions: &functions,
				parent: None
			}
		);
		let p = eval(Some(ast.unwrap().result), &env);
		match p {
			Ok(r) => println!("p: {:?}", r),
			Err(e) => panic!("ERROR: {}", e)
		}
	}

}

fn tokenize(program: &str) -> Vec<String>
{
	let mut iterator = Box::new(program.chars());
	let count = iterator.clone().count();
	let mut vec:Vec<char> = Vec::with_capacity(count);

	// println!("{:?}", iterator2);

	loop {
		match iterator.next() {
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
		let token = tokens.remove(0);

		if token == "(" {
			let mut vec:Vec<AST> = vec![];
			let mut tmp_tokens = tokens.clone();

			while tmp_tokens[0] != ")" {
				match read_from_tokens(tmp_tokens.clone()) {
					Ok(data) => {
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
	let to_int = token.parse::<u64>();
	let to_float = token.parse::<f64>();

	if to_int.is_ok() {
		AST::Integer(to_int.unwrap_or_default())
	} else if to_float.is_ok() {
		AST::Float(to_float.unwrap_or_default())
	} else {
		AST::Symbol(token.to_string())
	}
}

fn eval(ast_option: Option<AST>, env: &RefCell<Env>) -> Result<Option<AST>, &'static str> {
	match ast_option {
		Some(ast) => {
			println!("{:?}", ast);

			if let AST::Symbol(s) = ast {
				match env.borrow_mut().get(&s) {
					Some(DataType::Integer(i)) => Ok(Some(AST::Integer(i))),
					Some(DataType::Float(f)) => Ok(Some(AST::Float(f))),
					Some(DataType::Symbol(ref ss)) => Ok(Some(AST::Symbol(ss.clone()))),
					None => panic!("'{}' is not defined", s.to_string())
				}
			}
			else if let AST::Children(list) = ast {
				let solved_list: Vec<Option<AST>> = {
					let has_children = list.iter().any(|x| if let AST::Children(_) = *x { true } else { false });
					if has_children {
						list.into_iter().map(|x| eval(Some(x), &env).unwrap() ).collect::<_>()
					} else {
						list.into_iter().map(|x| Some(x)).collect::<_>()
					}
				};

				if let Some(AST::Symbol(ref s0)) = solved_list[0] {
					if s0 == "define" {
						if let Some(AST::Symbol(ref s1)) = solved_list[1].clone() {
							match Some(solved_list[2].clone()) {
								Some(Some(AST::Integer(i))) => { env.borrow_mut().local.borrow_mut().insert(s1.clone(), DataType::Integer(i)); },
								Some(Some(AST::Float(f))) => { env.borrow_mut().local.borrow_mut().insert(s1.clone(), DataType::Float(f)); },
								Some(Some(AST::Symbol(ref s))) => {env.borrow_mut().local.borrow_mut().insert(s1.clone(), DataType::Symbol(s.clone())); },
								Some(Some(AST::Children(_))) => { return Err("should not reach here"); },
								Some(None) | None => { }
							};
							return Ok(None)
						} else {
							return Err("definition name must be a symbol");
						}
					} else {
						// println!("{:?}", env.borrow());
						Ok(solved_list[0].clone())
					}
				} else {
					// println!("{:?}", env.borrow());
					Ok(solved_list[0].clone())
				}
			} else {
				// println!("{:?}", env.borrow());
				Ok(Some(ast))
			}
		},
		None => {
			Ok(None)
  		}
	}

}
