;; Fibonacci — tree recursion (exponential)
;; (fib 25) = 75025
(begin
  (print "fibonacci example")
  (define fib
    (lambda (n)
      (if (< n 2)
        n
        (+ (fib (- n 1)) (fib (- n 2))))))
  (display (fib 10))
  (newline)
  (display (fib 25))
  (newline))
