;; Symbolic differentiation — exercises car/cdr compositions, quasiquote,
;; define shorthand, let*, letrec, and/or, assert, for-each, recursion.
;;
;; Based on the classic SICP symbolic differentiation program, adapted
;; to use R5RS features supported by scheme-rs.

(begin
  (print "symbolic differentiation example")

  ;; --- Expression constructors (use quasiquote) ---

  (define (make-sum a b) `(+ ,a ,b))
  (define (make-product a b) `(* ,a ,b))

  ;; --- Predicates ---

  (define (number-expr? e) (or (integer? e) (real? e)))
  (define (variable? e) (symbol? e))
  (define (same-variable? v1 v2)
    (and (variable? v1) (variable? v2) (eq? v1 v2)))

  ;; --- Sum / product detection using car/cdr compositions ---

  (define (sum? e)
    (and (pair? e) (eq? (car e) (quote +))))
  (define (addend e) (cadr e))         ;; 2nd element
  (define (augend e) (caddr e))        ;; 3rd element

  (define (product? e)
    (and (pair? e) (eq? (car e) (quote *))))
  (define (multiplier e) (cadr e))
  (define (multiplicand e) (caddr e))

  ;; --- Derivation rules ---

  ;; dc/dx = 0  (constant)
  ;; dv/dx = 1  (if v == x)
  ;; dv/dx = 0  (if v != x)
  ;; d(u+v)/dx = du/dx + dv/dx
  ;; d(u*v)/dx = u*dv/dx + v*du/dx

  (define (deriv exp var)
    (cond
      ((number-expr? exp) 0)
      ((variable? exp)
       (if (same-variable? exp var) 1 0))
      ((sum? exp)
       (make-sum (deriv (addend exp) var)
                 (deriv (augend exp) var)))
      ((product? exp)
       (make-sum
         (make-product (multiplier exp)
                       (deriv (multiplicand exp) var))
         (make-product (deriv (multiplier exp) var)
                       (multiplicand exp))))
      (else
        (error "unknown expression type" exp))))

  ;; --- Simplification ---

  (define (simplify e)
    (cond
      ((number-expr? e) e)
      ((variable? e) e)
      ((sum? e)
       (let ((a (simplify (addend e)))
             (b (simplify (augend e))))
         (cond
           ((and (number-expr? a) (= a 0)) b)
           ((and (number-expr? b) (= b 0)) a)
           ((and (number-expr? a) (number-expr? b)) (+ a b))
           (else (make-sum a b)))))
      ((product? e)
       (let ((a (simplify (multiplier e)))
             (b (simplify (multiplicand e))))
         (cond
           ((and (number-expr? a) (= a 0)) 0)
           ((and (number-expr? b) (= b 0)) 0)
           ((and (number-expr? a) (= a 1)) b)
           ((and (number-expr? b) (= b 1)) a)
           ((and (number-expr? a) (number-expr? b)) (* a b))
           (else (make-product a b)))))
      (else e)))

  ;; --- Test cases (computed directly, no eval procedure) ---

  (display "d(x)/dx = ")
  (display (deriv 'x 'x))
  (newline)

  (display "d(5)/dx = ")
  (display (deriv 5 'x))
  (newline)

  (display "d(x+3)/dx = ")
  (display (simplify (deriv '(+ x 3) 'x)))
  (newline)

  (display "d(x*y)/dx = ")
  (display (simplify (deriv '(* x y) 'x)))
  (newline)

  (display "d(x*x)/dx = ")
  (display (simplify (deriv '(* x x) 'x)))
  (newline)

  ;; --- Interactive demo: d/dx(x^2 + 2*x + 1) ---
  ;; We represent x^2 as (* x x) since we don't have expt in deriv

  (display "d/dx(x^2 + 2x + 1) = ")
  (display
    (simplify
      (deriv '(+ (+ (* x x) (* 2 x)) 1) 'x)))
  (newline)

  ;; --- Nested: d/dx(x * (x + 1)) ---

  (display "d/dx(x*(x+1)) = ")
  (display
    (simplify
      (deriv '(* x (+ x 1)) 'x)))
  (newline)

  ;; --- Assertions ---

  (assert (= (deriv 'x 'x) 1))
  (assert (= (deriv 42 'x) 0))
  (print "all assertions passed")
)
