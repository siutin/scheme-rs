#[macro_use]
extern crate log;
extern crate env_logger;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::io;
use std::io::Write;

macro_rules! tuplet {
 { ($y:ident $(, $x:ident)*) = $v:expr } => {
    let ($y,$($x),*, _) = tuplet!($v ; 1 ; ($($x),*) ; ($v.get(0)) ); };
 { ($y:ident , * $x:ident) = $v:expr } => {
    let ($y,$x) = tuplet!($v ; 1 ; () ; ($v.get(0)) ); };
 { ($y:ident $(, $x:ident)* , * $z:ident) = $v:expr } => {
    let ($y,$($x),*, $z) = tuplet!($v ; 1 ; ($($x),*) ; ($v.get(0)) ); };
 { $v:expr ; $j:expr ; ($y:ident $(, $x:ident)*) ; ($($a:expr),*)  } => {
    tuplet!( $v ; $j+1 ; ($($x),*) ; ($($a),*,$v.get($j)) ) };
 { $v:expr ; $j:expr ; () ; ($($a:expr),*) } => {
   {
    if $v.len() >= $j {
        let remain = $v.len() - $j;
        if remain > 0 {
            ($($a),*, Some(&$v[$j..]))
        } else {
            ($($a),*, None)
        }
    } else {
        ($($a),*, None)
    }
   }
 }
}

#[derive(Clone, Debug)]
enum AST {
    Integer(i64),
    Float(f64),
    Symbol(String),
    Children(Vec<AST>)
}

#[derive(Debug)]
struct ReadFromTokenResult {
    remain: Vec<String>,
    result: AST
}

struct Function(pub Rc<Fn(Vec<DataType>) -> Result<Option<DataType>, &'static str>>);

impl Function {
    fn call(&self, arguments: Vec<DataType>) -> Result<Option<DataType>, &'static str> {
        (self.0)(arguments)
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function(self.0.clone())
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw = &self.0 as *const _;
        f.debug_tuple("Function").field(&raw).finish()
    }
}

#[derive(Clone, Debug)]
enum Number {
    Integer(i64),
    Float(f64),
}

impl From<Number> for f64 {
    fn from(value: Number) -> f64 {
        match value {
            Number::Float(f) => f,
            Number::Integer(i) => i as f64,
            _ => std::f64::NAN
        }
    }
}

impl From<Number> for i64 {
    fn from(value: Number) -> i64 {
        match value {
            Number::Integer(i) => i,
            Number::Float(f) => f as i64,
            _ => 0
        }
    }
}

impl Into<Number> for f64 {
    fn into(self) -> Number {
        Number::Float(self)
    }
}

impl Into<Number> for i64 {
    fn into(self) -> Number {
        Number::Integer(self)
    }
}

#[derive(Clone, Debug)]
enum DataType {
    Number(Number),
    Symbol(String),
    Proc(Function),
    List(Vec<DataType>)
}

#[derive(Debug)]
#[derive(Clone)]
struct Env<'a> {
    local: &'a RefCell<HashMap<String, DataType>>,
    parent: Option<Box<RefCell<Env<'a>>>>
}

impl<'a> Env<'a> {
    fn get(&self, key: &String) -> Option<DataType> {
        match self.local.borrow().get::<str>(key) {
            Some(&DataType::Number(Number::Integer(i))) => Some(DataType::Number(Number::Integer(i))),
            Some(&DataType::Number(Number::Float(f))) => Some(DataType::Number(Number::Float(f))),
            Some(&DataType::Symbol(ref ss)) => Some(DataType::Symbol(ss.clone())),
            Some(&DataType::Proc(ref p)) => Some(DataType::Proc(p.clone())),
            Some(&DataType::List(ref l)) => Some(DataType::List(l.clone())),
            None => {
                match self.parent {
                    Some(ref some_parent) => {
                        let parent_borrow = some_parent.borrow();
                        parent_borrow.get(key)
                    }
                    None => None
                }
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    let local = RefCell::new(setup());

    let mut env = Env {
        local: &local,
        parent: None
    };
    debug!("Env: {:?}", env);

    println!("Welcome to scheme-rs");
    repl(&mut env);
}

fn repl(mut env: &mut Env) {
    loop {
        print!("scheme=> ");
        io::stdout().flush().expect("cannot flush screen");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("cannot read input");
        match parse(input.as_str()).and_then(|ast| eval(Some(ast.result), &mut env)) {
            Ok(Some(d)) => println!("{:?}", d),
            Ok(None) => {}
            Err(e) => println!("error: {}", e)
        }
    }
}

fn parse(program: &str) -> Result<ReadFromTokenResult, &'static str> {
    debug!("program: {}", program);
    let tokens = tokenize(program);
    debug!("tokens: {:?}", tokens);
    let ast = read_from_tokens(tokens.clone());
    debug!("ast: {:?}", ast);
    return ast;
}

fn tokenize(program: &str) -> Vec<String>
{
    let iterator = Box::new(program.chars());
    let count = iterator.clone().count();
    let vec = iterator.fold(Vec::with_capacity(count), |mut acc, x| {
        if x == '(' || x == ')' {
            acc.extend(vec![' ', x, ' '])
        } else {
            acc.push(x)
        }
        acc
    });
    let s: String = vec.into_iter().collect();
    let ss: Vec<String> = s.split_whitespace().map(|x| x.to_string()).collect();
    ss
}

fn read_from_tokens(mut tokens: Vec<String>) -> Result<ReadFromTokenResult, &'static str> {
    if tokens.len() > 0 {
        let token = tokens.remove(0);

        if token == "(" {
            let mut vec: Vec<AST> = vec![];
            let mut tmp_tokens = tokens.clone();

            if tmp_tokens.is_empty() {
                return Err("syntax error");
            }

            while !tmp_tokens.is_empty() && tmp_tokens.first().unwrap() != ")" {
                match read_from_tokens(tmp_tokens.clone()) {
                    Ok(data) => {
                        vec.push(data.result);
                        tmp_tokens = data.remain.clone();
                    }
                    Err(e) => { return Err(e); }
                }
            }
            if tmp_tokens.is_empty() {
                return Err("syntax error");
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

fn eval(ast_option: Option<AST>, env: &mut Env) -> Result<Option<DataType>, &'static str> {
    debug!("eval");
    debug!("{:?}", ast_option);
    match ast_option.clone() {
        Some(AST::Symbol(s)) => {
            debug!("ast is a symbol: {:?}", s);
            match env.get(&s) {
                Some(data) => Ok(Some(data)),
                None => Err("symbol is not defined.")
            }
        }
        Some(AST::Children(list)) => {
            debug!("ast is a children: {:?}", list);

            if list.is_empty() {
                return Err("syntax error");
            }

            tuplet!((s0,s1,s2) = list);

            if let Some(&AST::Symbol(ref s0)) = s0 {
                match s0.as_str() {
                    "define" => {
                        if let (Some(&AST::Symbol(ref s1)), Some(&ref a2)) = (s1, s2) {
                            match a2.clone() {
                                AST::Integer(i) => { env.clone().local.borrow_mut().insert(s1.clone(), DataType::Number(Number::Integer(i))); }
                                AST::Float(f) => { env.clone().local.borrow_mut().insert(s1.clone(), DataType::Number(Number::Float(f))); }
                                AST::Symbol(ref s) => { env.clone().local.borrow_mut().insert(s1.clone(), DataType::Symbol(s.clone())); }
                                AST::Children(_) => unimplemented!()
                            }
                            return Ok(None);
                        }
                        return Err("wrong syntax for define expression");
                    }
                    _ => {
                        debug!("Some(AST::Symbol) but not define");
                        debug!("proc_key : {}", s0);
                        let env_shared = env.clone();
                        let data_option = match env_shared.get(s0) {
                            Some(d) => Some(d.clone()),
                            None => None
                        };

                        match data_option {
                            Some(DataType::Proc(ref f)) => {
                                let slice = &list[1..list.len()];
                                let args = slice.iter()
                                    .map(|x| eval(Some(x.clone()), &mut env_shared.clone()))
                                    .filter_map(|r| r.ok())
                                    .filter(|x| x.is_some())
                                    .flat_map(|x| x)
                                    .collect::<Vec<DataType>>();

                                f.call(args).and_then(|r| {
                                    match r {
                                        Some(data) => Ok(Some(data)),
                                        None => Ok(None)
                                    }
                                })
                            }
                            Some(_) | None => Err("Symbol is not defined.")
                        }
                    }
                }
            } else {
                Err("syntax error")
            }
        }
        Some(_) | None => {
            debug!("ast is not a symbol/children");
            let data = match ast_option {
                Some(AST::Integer(i)) => Some(DataType::Number(Number::Integer(i))),
                Some(AST::Float(f)) => Some(DataType::Number(Number::Float(f))),
                Some(_) => unreachable!(),
                None => None
            };
            Ok(data)
        }
    }
}

fn setup() -> HashMap<String, DataType> {
    let mut map = HashMap::new();
    map.insert("pi".to_string(), DataType::Number(Number::Float(std::f64::consts::PI)));

    // pre-defined commands
    map.insert("begin".to_string(), DataType::Proc(Function(Rc::new(|mut vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "begin", vec);
        Ok(vec.pop().clone())
    }))));

    map.insert("print".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "hello", vec);
        if vec.len() != 1 {
            return Err("print function requires one argument only");
        }

        let value_option = vec.first();
        if value_option.is_none() {
            return Err("unknown argument type");
        }
        println!("{}",datatype2str(value_option.unwrap()));
//        print_fn(value_option.unwrap());
        Ok(None)
    }))));

    map.insert("*".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "*", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });
        if !is_numbers {
            return Err("wrong argument datatype");
        }

        let desc = vec.iter().map(|&ref x|
            match x {
                &DataType::Number(Number::Integer(i)) => i.to_string(),
                &DataType::Number(Number::Float(f)) => f.to_string(),
                _ => panic!("Something went wrong"),
            }
        ).collect::<Vec<String>>().join(" x ");
        debug!("Description: {}", desc);

        let is_integers = vec.iter().all(|&ref x| if let &DataType::Number(Number::Integer(_)) = x { true } else { false });
        let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });
        if is_integers {
            let data: i64 = numbers.map(|x| {
                let y: i64 = x.clone().into();
                y
            }).product();
            Ok(Some(DataType::Number(data.into())))
        } else {
            let data: f64 = numbers.map(|x| {
                let y: f64 = x.clone().into();
                y
            }).product();
            Ok(Some(DataType::Number(data.into())))
        }
    }))));

    map.insert("+".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "+", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });
        if !is_numbers {
            return Err("wrong argument datatype");
        }

        let desc = vec.iter().map(|&ref x|
            match x {
                &DataType::Number(Number::Integer(i)) => i.to_string(),
                &DataType::Number(Number::Float(f)) => f.to_string(),
                _ => unreachable!(),
            }
        ).collect::<Vec<String>>().join(" + ");
        debug!("Description: {}", desc);

        let is_integers = vec.iter().all(|&ref x| if let &DataType::Number(Number::Integer(_)) = x { true } else { false });
        let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });
        if is_integers {
            let data: i64 = numbers.map(|x| {
                let y: i64 = x.clone().into();
                y
            }).sum();
            Ok(Some(DataType::Number(data.into())))
        } else {
            let data: f64 = numbers.map(|x| {
                let y: f64 = x.clone().into();
                y
            }).sum();
            Ok(Some(DataType::Number(data.into())))
        }
    }))));

    map.insert("-".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "+", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });

        if !is_numbers {
            return Err("wrong argument datatype");
        }

        let is_integers = vec.iter().all(|&ref x| if let &DataType::Number(Number::Integer(_)) = x { true } else { false });

        let desc = vec.iter().map(|&ref x|
            match x {
                &DataType::Number(Number::Integer(i)) => i.to_string(),
                &DataType::Number(Number::Float(f)) => f.to_string(),
                _ => unreachable!(),
            }
        ).collect::<Vec<String>>().join(" - ");
        debug!("Description: {}", desc);

        let numbers = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } });
        let data = if is_integers {
            let value: i64 = numbers.map(|x| {
                let y: i64 = x.clone().into();
                y
            }).sum();
            Number::Integer(value * -1)
        } else {
            let value: f64 = numbers.map(|x| {
                let y: f64 = x.clone().into();
                y
            }).sum();
            Number::Float(value * -1.0)
        };
        Ok(Some(DataType::Number(data)))
    }))));

    map.insert("/".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "/", vec);
        let is_numbers = vec.iter().all(|&ref x| if let &DataType::Number(_) = x { true } else { false });

        if !is_numbers {
            return Err("wrong argument datatype");
        }

        let desc = vec.iter().map(|&ref x|
            match x {
                &DataType::Number(Number::Integer(i)) => i.to_string(),
                &DataType::Number(Number::Float(f)) => f.to_string(),
                _ => unreachable!(),
            }
        ).collect::<Vec<String>>().join(" / ");
        debug!("Description: {}", desc);

        let value: f64 = vec.iter().filter_map(|&ref x| { if let &DataType::Number(ref y) = x { Some(y.clone()) } else { None } })
            .map(|x| {
                let y: f64 = x.clone().into();
                y
            })
            .fold(0.0, |mut acc, x| {
                if acc == 0.0 { acc = x; } else { acc = acc / x; }
                acc
            });
        Ok(Some(DataType::Number(Number::Float(value))))
    }))));

    map.insert("list".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "list", vec);
        Ok(Some(DataType::List(vec)))
    }))));

    map.insert("car".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "car", vec);
        if vec.len() != 1 {
            return Err("car function requires one argument only");
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("car function unknown argument type");
        }
        match value_option.unwrap() {
            &DataType::List(ref vec) => {
                let value = vec.first();
                if value.is_some() {
                    Ok(Some(DataType::from(value.unwrap().clone())))
                } else {
                    Err("car function requires a non-empty list")
                }
            },
            _ => Err("car function requires an argument of type 'list'")
        }
    }))));

    map.insert("cdr".to_string(), DataType::Proc(Function(Rc::new(|vec: Vec<DataType>| {
        debug!("Function - name: {:?} - Args: {:?}", "cdr", vec);
        if vec.len() != 1 {
            return Err("cdr function requires one argument only");
        }
        let value_option = vec.first();
        if value_option.is_none() {
            return Err("cdr function unknown argument type");
        }
        match value_option.unwrap() {
            &DataType::List(ref vec) => {
                if vec.len() > 0 {
                    Ok(Some(DataType::List((&vec[1..]).to_vec())))
                } else {
                    Err("cdr function requires a non-empty list")
                }
            },
            _ => Err("cdr function requires an argument of type 'list'")
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

    return map;
}

fn datatype2str(value: &DataType) -> String {
    match value {
        &DataType::Number(Number::Integer(i)) => format!("{}", i),
        &DataType::Number(Number::Float(f)) => format!("{}", f),
        &DataType::Symbol(ref s) => format!("{}", s),
        &DataType::Proc(ref p) => format!("{:?}", p),
        &DataType::List(ref v) => format!("'({})", v.iter()
            .map(|d|datatype2str(d)).collect::<Vec<_>>().join(" "))
    }
}