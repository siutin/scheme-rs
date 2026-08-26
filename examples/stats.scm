;; Statistics and numerical math — exercises transcendental functions,
;; filter, for-each, named let, do loops, assert, integer?/real?, and
;; string->number/number->string.

(begin
  (print "statistics and numerical math example")

  ;; --- Basic statistics ---

  (define (mean lst)
    (/ (let loop ((lst lst) (sum 0))
         (if (null? lst) sum (loop (cdr lst) (+ sum (car lst)))))
       (length lst)))

  (define (square x) (* x x))

  ;; Variance = mean of squared deviations
  (define (variance lst)
    (let ((mu (mean lst)))
      (mean
        (map (lambda (x) (square (- x mu))) lst))))

  ;; Standard deviation = sqrt(variance)
  (define (stddev lst)
    (sqrt (variance lst)))

  (define data (list 2 4 4 4 5 5 7 9))

  (display "data: ")
  (display data)
  (newline)

  (display "mean: ")
  (display (mean data))
  (newline)

  (display "variance: ")
  (display (variance data))
  (newline)

  (display "stddev: ")
  (display (stddev data))
  (newline)

  ;; --- Transcendental functions ---

  (display "exp(1) = ")
  (display (exp 1))
  (newline)

  (display "log(e) = ")
  (display (log (exp 1)))
  (newline)

  (display "sin(pi/2) = ")
  (display (sin (/ pi 2)))
  (newline)

  (display "cos(0) = ")
  (display (cos 0))
  (newline)

  (display "tan(pi/4) = ")
  (display (tan (/ pi 4)))
  (newline)

  ;; atan2 — angle of (1,1) vector
  (display "atan(1,1) = ")
  (display (atan 1 1))
  (newline)

  ;; --- Number/String conversion ---

  (display "string->number \"42\": ")
  (display (string->number "42"))
  (newline)

  (display "string->number \"3.14\": ")
  (display (string->number "3.14"))
  (newline)

  (display "number->string 255 (hex): ")
  (display (number->string 255 16))
  (newline)

  (display "number->string 255 (binary): ")
  (display (number->string 255 2))
  (newline)

  ;; --- filter + for-each: process numbers ---

  (define all-nums (list 1 2 3 4 5 6 7 8 9 10))

  (display "evens: ")
  (display (filter even? all-nums))
  (newline)

  (display "primes: ")
  (display (filter
             (lambda (n)
               (and (> n 1)
                    (let loop ((d 2))
                      (cond
                        ((> (* d d) n) #t)
                        ((= 0 (modulo n d)) #f)
                        (else (loop (+ d 1)))))))
             all-nums))
  (newline)

  ;; --- do loop: compute factorial ---

  (display "10! = ")
  (display
    (do ((i 1 (+ i 1))
         (acc 1 (* acc i)))
        ((> i 10) acc)))
  (newline)

  ;; --- let* for sequential bindings ---

  (display "let* chain: ")
  (display
    (let* ((x 2)
           (y (+ x 1))
           (z (+ y 1)))
      (* x y z)))
  (newline)

  ;; --- letrec for mutual recursion ---

  (display "letrec (even?/odd?): ")
  (display
    (letrec ((my-even? (lambda (n) (if (= n 0) #t (my-odd? (- n 1)))))
             (my-odd?  (lambda (n) (if (= n 0) #f (my-even? (- n 1))))))
      (my-even? 10)))
  (newline)

  ;; --- Assertions ---

  (assert (real? (mean data)))
  (assert (integer? (string->number "42")))
  (assert (= 5 (length (filter even? all-nums))))
  (assert (= 4 (length (filter even? (list 1 2 3 4 5 6 7 8)))))
  (print "all assertions passed")
)
