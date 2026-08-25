// RUN: cargo bench --features "unstable"

#![cfg_attr(feature = "unstable", feature(test))]
#[cfg(all(feature = "unstable", test))]
extern crate test;
#[cfg(all(feature = "unstable", test))]
mod bench {
    use test::Bencher;
    use std::cell::RefCell;
    use std::rc::Rc;
    use scheme_rs::{eval, parse, setup, Env, DataType, SchemeError};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[bench]
    fn fact10_bench(b: &mut Bencher) {
        let env_ref = default_env();
        run_with_env("(define fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))", env_ref.clone());

        b.iter(|| {
            run_with_env("(fact 20)", env_ref.clone());
        })
    }

    #[derive(Debug)]
    struct TestResult {
        value: Result<Option<DataType>, SchemeError>,
        env: Rc<RefCell<Env>>
    }

    fn default_env() -> Rc<RefCell<Env>> {
        let local = Box::new(RefCell::new(setup()));
        let env = Env {
            local,
            parent: None
        };

        let env_ref = Rc::new(RefCell::new(env));
        env_ref
    }

    fn run_with_env(s: &str, env_ref: Rc<RefCell<Env>>) -> TestResult {
        let result = parse(s)
            .and_then(|ast| eval(Some(ast.result), env_ref.clone()));

        TestResult {
            value: result.clone(),
            env: env_ref.clone()
        }
    }
}
