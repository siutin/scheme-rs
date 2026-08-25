// RUN: cargo bench --features "unstable"

#![cfg_attr(feature = "unstable", feature(test))]
#[cfg(all(feature = "unstable", test))]
extern crate test;
#[cfg(all(feature = "unstable", test))]
mod bench {
    use test::Bencher;
    use std::cell::RefCell;
    use std::rc::Rc;
    use scheme_rs::{eval, parse, setup, Env};

    fn default_env() -> Rc<RefCell<Env>> {
        let local = Box::new(RefCell::new(setup()));
        let env = Env { local, parent: None };
        Rc::new(RefCell::new(env))
    }

    fn run(s: &str, env: Rc<RefCell<Env>>) {
        parse(s).and_then(|ast| eval(Some(ast.result), env)).unwrap();
    }

    // Tree recursion: factorial (non-tail)
    #[bench]
    fn fact20_bench(b: &mut Bencher) {
        let env = default_env();
        run("(define fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))", env.clone());
        b.iter(|| { run("(fact 20)", env.clone()); })
    }

    // Exponential tree recursion: fibonacci
    #[bench]
    fn fib25_bench(b: &mut Bencher) {
        let env = default_env();
        run("(define fib (lambda (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))))", env.clone());
        b.iter(|| { run("(fib 25)", env.clone()); })
    }

    // Pure TCO loop: 100k iterations of decrement + compare
    #[bench]
    fn tco_loop_100k_bench(b: &mut Bencher) {
        let env = default_env();
        run("(define loop (lambda (n) (if (= n 0) (quote done) (loop (- n 1)))))", env.clone());
        b.iter(|| { run("(loop 100000)", env.clone()); })
    }

    // Tail-recursive accumulation: sum 1..1000
    #[bench]
    fn sum_to_1000_bench(b: &mut Bencher) {
        let env = default_env();
        run("(define sum-to (lambda (n acc) (if (= n 0) acc (sum-to (- n 1) (+ acc n)))))", env.clone());
        b.iter(|| { run("(sum-to 1000 0)", env.clone()); })
    }

    // Deep nested recursion: Ackermann (2,8)
    #[bench]
    fn ackermann_2_8_bench(b: &mut Bencher) {
        let env = default_env();
        run(r#"
        (define ack
          (lambda (m n)
            (cond
              ((= m 0) (+ n 1))
              ((= n 0) (ack (- m 1) 1))
              (else (ack (- m 1) (ack m (- n 1)))))))
        "#, env.clone());
        b.iter(|| { run("(ack 2 8)", env.clone()); })
    }

    // List operations: build a list of 1000 elements, then sum it
    #[bench]
    fn list_ops_bench(b: &mut Bencher) {
        let env = default_env();
        run("(define sum (lambda (l) (if (null? l) 0 (+ (car l) (sum (cdr l))))))", env.clone());
        run("(define build (lambda (n acc) (if (= n 0) acc (build (- n 1) (cons n acc)))))", env.clone());
        b.iter(|| { run("(sum (build 1000 (quote ())))", env.clone()); })
    }
}
