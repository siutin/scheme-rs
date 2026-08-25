;; Factorial — tree recursion
;; (fact 20) = 2432902008176640000
(begin
  (print "factorial example")
  (define fact
    (lambda (n)
      (if (<= n 1)
        1
        (* n (fact (- n 1))))))
  (display (fact 10))
  (newline)
  (display (fact 20))
  (newline))
