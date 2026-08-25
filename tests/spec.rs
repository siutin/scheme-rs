use std::cell::RefCell;
use std::rc::Rc;
use scheme_rs::*;

#[test]
fn if_expression_test() {
    let test_result = run("(if (> (* 11 11) 120) #t #f)");
    assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
}

#[test]
fn quote_expression_test() {
    {
        let test_result = run("(quote apple)");
        assert_eq!(Ok(Some(DataType::Symbol("apple".to_string()))), test_result.value);
    }
    {
        let test_result = run("(quote \"orange\")");
        assert_eq!(Ok(Some(DataType::String("orange".to_string()))), test_result.value);
    }
    {
        let test_result = run("(quote 42)");
        assert_eq!(Ok(Some(DataType::Number(42.0))), test_result.value);
    }
    {
        let test_result = run("(quote #t)");
        assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
    }
    {
        let test_result = run("(quote (define x 1))");
        assert_eq!(Ok(Some(DataType::List(vec![
            DataType::Symbol("define".to_string()),
            DataType::Symbol("x".to_string()),
            DataType::Number(1.0),
        ]))), test_result.value);
    }
}

#[test]
fn quote_shorthand_test() {
    // 'symbol should work (no space)
    assert_eq!(Ok(Some(DataType::Symbol("foo".to_string()))), run("'foo").value);
    // '(1 2 3) should work — quote shorthand for lists
    assert_eq!(Ok(Some(DataType::List(vec![
        DataType::Number(1.0),
        DataType::Number(2.0),
        DataType::Number(3.0),
    ]))), run("'(1 2 3)").value);
    // '(a b c) — list of symbols
    assert_eq!(Ok(Some(DataType::List(vec![
        DataType::Symbol("a".to_string()),
        DataType::Symbol("b".to_string()),
        DataType::Symbol("c".to_string()),
    ]))), run("'(a b c)").value);
    // Nested quote shorthand
    assert_eq!(Ok(Some(DataType::List(vec![
        DataType::Number(1.0),
        DataType::List(vec![
            DataType::Number(2.0),
            DataType::Number(3.0),
        ]),
    ]))), run("'(1 (2 3))").value);
}

#[test]
fn variable_retrieving_test() {
    let test_result = run("(define r 10)(* pi (* r r))");
    assert_eq!(Ok(Some(DataType::Number(314.1592653589793))), test_result.value);
}

#[test]
fn lambda_retrieving_test() {
    let test_result = run(r#"
    (define circle-area (lambda (r) (* pi (* r r))))
    (circle-area 3)
    "#);
    assert_eq!(Ok(Some(DataType::Number(28.274333882308138))), test_result.value);
}

#[test]
fn recursive_lambda_test() {
    let test_result = run(r#"
    (define fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))
    (fact 10)
    "#);
    assert_eq!(Ok(Some(DataType::Number(3628800.0))), test_result.value);
}

#[test]
fn lambda_call_test() {
    let test_result = run(r#"
    (define twice (lambda (x) (* 2 x)))
    (twice 5)
    "#);
    assert_eq!(Ok(Some(DataType::Number(10.0))), test_result.value);
}

#[test]
fn nested_lambda_test() {
    {
        let test_result = run(r#"
    (define repeat (lambda (f) (lambda (x) (f (f x)))))
    repeat
    "#);
        if let Ok(Some(DataType::Lambda(_))) = test_result.value {
            assert!(true);
        } else {
            assert!(false);
        }
    }
    {
        let test_result = run(r#"
        (define add3
            (lambda (x y z)
                (+ ((lambda (x y)
                     (+ x y)) x y) z)))
        (add3 2 3 4)
        "#);
        assert_eq!(Ok(Some(DataType::Number(9.0))), test_result.value);
    }
}

#[test]
fn complex_lambda_test() {
    let test_result = run(r#"
    (define twice (lambda (x) (* 2 x)))
    (define repeat (lambda (f) (lambda (x) (f (f x)))))
    ((repeat (repeat twice)) 10)
    "#);
    assert_eq!(Ok(Some(DataType::Number(160.0))), test_result.value);
}

#[test]
fn unary_minus_test() {
    assert_eq!(Ok(Some(DataType::Number(-5.0))), run("(- 5)").value);
    assert_eq!(Ok(Some(DataType::Number(-5.0))), run("(- 5.0)").value);
    assert_eq!(Ok(Some(DataType::Number(7.0))), run("(- 10 3)").value);
    assert_eq!(Ok(Some(DataType::Number(5.0))), run("(- 10 3 2)").value);
    assert_eq!(Ok(Some(DataType::Number(-5.0))), run("(- 0 5)").value);
    assert_eq!(Ok(Some(DataType::Number(0.0))), run("(- 5 5)").value);
}

#[test]
fn division_by_zero_test() {
    assert_eq!(Err(SchemeError::DivisionByZero), run("(/ 1 0)").value);
    assert_eq!(Err(SchemeError::DivisionByZero), run("(/ 10 2 0)").value);
    assert_eq!(Err(SchemeError::DivisionByZero), run("(/ 100 0 5)").value);
    // Unary division should work: (/ 5) = 1/5 = 0.2
    assert_eq!(Ok(Some(DataType::Number(0.2))), run("(/ 5)").value);
    // Normal division still works
    assert_eq!(Ok(Some(DataType::Number(5.0))), run("(/ 10 2)").value);
    assert_eq!(Ok(Some(DataType::Number(2.0))), run("(/ 12 3 2)").value);
}

#[test]
fn lambda_fresh_env_test() {
    // Each lambda call must get a fresh env frame, not mutate the captured one.
    // If the closure env is mutated, the second call sees stale bindings.
    // All in one run (single begin block):
    let test_result = run(r#"
    (define f (lambda (x) x))
    (f 1)
    (f 2)
    "#);
    // begin returns the last value
    assert_eq!(Ok(Some(DataType::Number(2.0))), test_result.value);
}

#[test]
fn lambda_no_arg_leak_test() {
    // A lambda's param binding must not leak into the next call.
    // This tests the children-path call site (eval.rs line ~247).
    let test_result = run(r#"
    (define add (lambda (a b) (+ a b)))
    (add 3 4)
    (add 10 20)
    "#);
    assert_eq!(Ok(Some(DataType::Number(30.0))), test_result.value);
}

#[test]
fn lambda_via_apply_fresh_env_test() {
    // apply with a lambda must also use a fresh env frame.
    let test_result = run(r#"
    (define f (lambda (x) (* x x)))
    (apply f (list 3))
    (apply f (list 5))
    "#);
    assert_eq!(Ok(Some(DataType::Number(25.0))), test_result.value);
}

#[test]
fn lambda_via_map_fresh_env_test() {
    // map with a lambda must use a fresh env frame per element.
    let test_result = run(r#"
    (define double (lambda (x) (* 2 x)))
    (map double (list 1 2 3 4 5))
    "#);
    assert_eq!(
        Ok(Some(DataType::List(vec![
            DataType::Number(2.0),
            DataType::Number(4.0),
            DataType::Number(6.0),
            DataType::Number(8.0),
            DataType::Number(10.0),
        ]))),
        test_result.value
    );
}

#[test]
fn quote_string_type_test() {
    // Quoted strings should be DataType::String, not DataType::Symbol.
    let test_result = run("(quote \"hello world\")");
    assert_eq!(
        Ok(Some(DataType::String("hello world".to_string()))),
        test_result.value
    );
    // string? should return #t for quoted strings
    let test_result2 = run("(string? (quote \"test\"))");
    assert_eq!(Ok(Some(DataType::Bool(true))), test_result2.value);
}

#[test]
fn let_test() {
    // Basic let with two bindings
    let test_result = run("(let ((x 1) (y 2)) (+ x y))");
    assert_eq!(Ok(Some(DataType::Number(3.0))), test_result.value);
    // Let with empty bindings
    let test_result2 = run("(let () 42)");
    assert_eq!(Ok(Some(DataType::Number(42.0))), test_result2.value);
    // Let body can reference outer variables
    let test_result3 = run(r#"
    (define z 10)
    (let ((x 5)) (+ x z))
    "#);
    assert_eq!(Ok(Some(DataType::Number(15.0))), test_result3.value);
}

#[test]
fn tricky_test1 () {

    // Testing the case that the 1st element is a children and it returns a function/lambda after an evaluation
    // and then it would be evaluated again but without arguments

    // function
    let test_result1 = run("((begin +))");
    assert_eq!(Ok(Some(DataType::Number(0.0))), test_result1.value);

    // lambda
    let env_ref = default_env();
    run_with_env("(define add (lambda () (+)))", env_ref.clone());
    let test_result0 = run_with_env("((begin add))", env_ref.clone());
    assert_eq!(Ok(Some(DataType::Number(0.0))), test_result0.value);
}

#[test]
fn state_test() {
    let env_ref = default_env();
    let test_result0 = run_with_env("s", env_ref.clone());
    assert_eq!(Err("symbol is not defined.".into()), test_result0.value);

    let test_result1 = run_with_env("(define s \"hello world\")", env_ref.clone());
    assert_eq!(Ok(None), test_result1.value);

    let test_result2 = run_with_env("s", env_ref.clone());
    assert_eq!(Ok(Some(DataType::String("hello world".to_string()))), test_result2.value);
}

#[test]
fn type_test() {
    assert_eq!(Ok(Some(DataType::String("hello world".into()))), run("\"hello world\"").value);
    assert_eq!(Err("can not find an end quote".into()), run("\"hello world").value);
    assert_eq!(Ok(Some(DataType::Number(1.0))), run("1").value);
    assert_eq!(Ok(Some(DataType::Number(3.9))), run("3.9").value);
    assert_eq!(Ok(Some(DataType::Symbol("foo".into()))), run("'foo").value);
    assert_eq!(Ok(Some(DataType::Bool(true))), run("#t").value);
    assert_eq!(Err("syntax error".into()), run("#tt").value);
    assert_eq!(Ok(Some(DataType::Pair(
        (
            Box::new(DataType::Number(1.0)),
            Box::new(DataType::Number(2.0))
        )
    ))), run("(cons 1 2)").value);
    assert_eq!(Ok(Some(DataType::List(vec![
        DataType::Symbol("aa".into()),
        DataType::Symbol("bbb".into()),
        DataType::Symbol("cccc".into()),
    ]
    ))), run("(list 'aa 'bbb 'cccc)").value);
    if let Ok(Some(DataType::Proc(_))) = run("+").value { assert!(true) } else { assert!(false) }
    if let Ok(Some(DataType::Lambda(_))) = run("(lambda ()(print \"something\"))").value { assert!(true) } else { assert!(false) }
}

mod op {
    use super::*;

    #[test]
    fn stmt1() {
        let test_result = run("(+ 1 2 3 (+ 4 5) 6)");
        assert_eq!(Ok(Some(DataType::Number(21.0))), test_result.value);
    }

    #[test]
    fn stmt2() {
        let test_result = run("(- (/ (* 1 2 3 4 5) 6) 7)");
        assert_eq!(Ok(Some(DataType::Number(13.0))), test_result.value);
    }
}

mod std_function {
    use super::*;

    #[test]
    fn list() {
        let test_result = run("(list 0 1 2 3 0 0)");
        assert_eq!(Ok(Some(DataType::List(vec![
            DataType::Number(0.0),
            DataType::Number(1.0),
            DataType::Number(2.0),
            DataType::Number(3.0),
            DataType::Number(0.0),
            DataType::Number(0.0)
        ]))), test_result.value);
    }

    #[test]
    fn car() {
        let test_result = run("(car (list 0 1 2 3 0 0))");
        assert_eq!(Ok(Some(DataType::Number(0.0))), test_result.value);
    }

    #[test]
    fn cdr() {
        let test_result = run("(cdr (cdr (list 0 1 2 3 0 0)))");
        assert_eq!(Ok(Some(DataType::List(vec![
            DataType::Number(2.0),
            DataType::Number(3.0),
            DataType::Number(0.0),
            DataType::Number(0.0)
        ]))), test_result.value);
    }

    #[test]
    fn cons() {
        assert_eq!(Ok(Some(DataType::Pair(
            (
                Box::new(DataType::Number(1.0)),
                Box::new(DataType::Number(2.0))
            )
        ))), run("(cons 1 2)").value);
        assert_eq!(Err("cons function requires two argument only".into()), run("(cons 'a)").value);

    }

    #[test]
    fn abs() {
        let test_result = run("(abs -42)");
        assert_eq!(Ok(Some(DataType::Number(42.0))), test_result.value);
    }

    #[test]
    fn append() {
        assert_eq!(Ok(Some(DataType::List(vec![
            DataType::Number(1.0),
            DataType::Number(2.0),
            DataType::Number(3.0),
            DataType::Number(4.0),
            DataType::Number(5.0)
        ]))), run("(append (list 1 2 3) (list 4 5))").value);

        assert_eq!(Ok(Some(
            DataType::Pair(
                (
                    Box::new(
                        DataType::List(vec![
                            DataType::Number(1.0),
                            DataType::Number(2.0),
                            DataType::Number(3.0),
                        ])
                    ),
                    Box::new(DataType::Number(4.0))
                )
            )
        )), run("(append (list 1 2 3) 4)").value);

        assert_eq!(Ok(Some(
            DataType::Pair(
                (
                    Box::new(
                        DataType::List(vec![
                            DataType::Number(1.0),
                            DataType::Number(2.0),
                            DataType::Number(3.0),
                            DataType::Number(4.0),

                        ])
                    ),
                    Box::new(DataType::Bool(false))
                )
            )
        )), run("(append (list 1 2 3 4) #f)").value);

        assert_eq!(Ok(Some(
            DataType::Pair(
                (
                    Box::new(
                        DataType::List(vec![
                            DataType::Number(1.0),
                            DataType::Number(2.0)
                        ])
                    ),
                    Box::new(DataType::String("hello".into()))
                )
            )
        )), run("(append (list 1 2) \"hello\")").value);

        assert_eq!(Ok(Some(
            DataType::Pair(
                (
                    Box::new(
                        DataType::List(vec![
                            DataType::Number(1.0),
                            DataType::Number(2.0),
                            DataType::Number(3.0),
                        ])
                    ),
                    Box::new(DataType::Symbol("world".into()))
                )
            )
        )), run("(append (list 1 2 3) 'world)").value);

        // TODO: test append with procedure and lambda
    }

    #[test]
    fn apply() {
        {
            let test_result = run("(apply * (list 7 9))");
            assert_eq!(Ok(Some(DataType::Number(63.0))), test_result.value);
        }
        {
            let test_result = run("(apply (lambda (x y)(* x y)) (list 7 9))");
            assert_eq!(Ok(Some(DataType::Number(63.0))), test_result.value);
        }
    }

    #[test]
    fn length() {
        let test_result = run("(length (list 7 9 4 0 3))");
        assert_eq!(Ok(Some(DataType::Number(5.0))), test_result.value);
    }

    #[test]
    fn map() {
        assert_eq!(Ok(Some(
            DataType::List(vec![
                DataType::Bool(false),
                DataType::Bool(false),
                DataType::Bool(true),
                DataType::Bool(false),
                DataType::Bool(false),
                DataType::Bool(true),
                DataType::Bool(false),
            ])
        )), run("(map number? (list #t \"hello\" 3 's - 2.1 (lambda () (+ 1 2)) ))").value);

        assert_eq!(Ok(Some(
            DataType::List(vec![
                DataType::Number(1.0),
                DataType::Number(4.0),
                DataType::Number(9.0),
                DataType::Number(16.0),
                DataType::Number(25.0),
            ])
        )), run("(map (lambda (x) (* x x)) (list 1 2 3 4 5))").value);

        assert_eq!(Ok(Some(
            DataType::List(vec![
                DataType::Pair(
                    (
                        Box::new(DataType::Number(2.0)),
                        Box::new(DataType::Number(1.0))
                    )
                ),
                DataType::Pair(
                    (
                        Box::new(DataType::Number(4.0)),
                        Box::new(DataType::Number(3.0))
                    )
                )
            ])
        )), run(r#"(map (lambda (x)
                                   (cons (car (cdr x))
                                   (car x) ))
                           (list (list 1 2) (list 3 4)))"#).value);

        {
            let env_ref = default_env();
            run_with_env("(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))", env_ref.clone());

            assert_eq!(Ok(Some(DataType::List(
                vec![
                    DataType::Number(1.0),
                    DataType::Number(1.0),
                    DataType::Number(2.0),
                    DataType::Number(3.0),
                    DataType::Number(5.0),
                    DataType::Number(8.0),
                    DataType::Number(13.0),
                    DataType::Number(21.0),
                    DataType::Number(34.0),
                    DataType::Number(55.0)
                ]
            ))), run_with_env("(map fib (list 0 1 2 3 4 5 6 7 8 9))", env_ref.clone()).value);
        }
    }

    #[test]
    fn max_min() {
        {
            let test_result = run("(max 7 9 4 0 3)");
            assert_eq!(Ok(Some(DataType::Number(9.0))), test_result.value);
        }
        {
            let test_result = run("(min 7 9 4 0 3)");
            assert_eq!(Ok(Some(DataType::Number(0.0))), test_result.value);
        }
    }

    #[test]
    fn not() {
        {
            let test_result = run("(not #t)");
            assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
        }
        {
            let test_result = run("(not #f)");
            assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
        }
        {
            let test_result = run("(not 1)");
            assert_eq!(Err("not function requires an argument of type 'boolean'".into()), test_result.value);
        }
    }

    mod type_checking_function {
        use super::*;

        #[test]
        fn list_q() {
            {
                let test_result = run("(list? (list 7 9 4 0 3))");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(list? 1)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(list? 5.5)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(list? \"hello\")");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(list? 'hello)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(list? +)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(list? (lambda (x y) (+ x y)))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(list? (cons 1 2))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
        }

        #[test]
        fn number_q() {
            {
                let test_result = run("(number? (list 7 9 4 0 3))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(number? 1)");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(number? 5.5)");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(number? \"hello\")");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(number? 'hello)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(number? +)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(number? (lambda (x y) (+ x y)))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(number? (cons 1 2))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
        }

        #[test]
        fn pair_q() {
            {
                let test_result = run("(pair? (list 7 9 4 0 3))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? 1)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? 5.5)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? \"hello\")");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? 'hello)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? +)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? (lambda (x y) (+ x y)))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(pair? (cons 1 2))");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
        }

        #[test]
        fn procedure_q() {
            {
                let test_result = run("(procedure? (list 7 9 4 0 3))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(procedure? 1)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(procedure? 5.5)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(procedure? \"hello\")");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(procedure? 'hello)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(procedure? +)");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(procedure? (lambda (x y) (+ x y)))");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(procedure? (cons 1 2))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
        }

        #[test]
        fn string_q() {
            {
                let test_result = run("(string? (list 7 9 4 0 3))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(string? 1)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(string? 5.5)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(string? \"hello\")");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(string? 'hello)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(string? +)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(string? (lambda (x y) (+ x y)))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(string? (cons 1 2))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
        }

        #[test]
        fn symbol_q() {
            {
                let test_result = run("(symbol? (list 7 9 4 0 3))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(symbol? 1)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(symbol? 5.5)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(symbol? \"hello\")");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(symbol? 'hello)");
                assert_eq!(Ok(Some(DataType::Bool(true))), test_result.value);
            }
            {
                let test_result = run("(symbol? +)");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(symbol? (lambda (x y) (+ x y)))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
            {
                let test_result = run("(symbol? (cons 1 2))");
                assert_eq!(Ok(Some(DataType::Bool(false))), test_result.value);
            }
        }
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    env_logger::init();
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

fn run(s: &str) -> TestResult {
    run_with_env(s, default_env().clone())
}

fn run_with_env(s: &str, env_ref: Rc<RefCell<Env>>) -> TestResult {
    let result = parse(s)
        .and_then(|ast| eval(Some(ast.result), env_ref.clone()));

    TestResult {
        value: result.clone(),
        env: env_ref.clone()
    }
}
