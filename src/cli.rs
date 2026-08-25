use std::io;
use std::io::Write;
use log::debug;
use scheme_rs::*;

fn main() {
    env_logger::init();
    let env = Env::root(setup());
    debug!("Env: {:?}", env);

    println!("Welcome to scheme-rs");
    repl(env);
}

fn repl(env: EnvRef) {
    loop {
        print!("scheme=> ");
        io::stdout().flush().expect("cannot flush screen");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("cannot read input");
        match parse(input.as_str()).and_then(|ast| eval(Some(ast.result), env.clone())) {
            Ok(Some(d)) => println!("{:?}", d),
            Ok(None) => {}
            Err(e) => println!("error: {}", e)
        }
        debug!("ENV: {:?}", &env);
    }
}
