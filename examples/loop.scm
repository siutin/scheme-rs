;; Tail-recursive loop — demonstrates TCO
;; Counts down from 100000 without stack overflow
(begin
  (print "tail-call loop example")
  (define loop
    (lambda (n)
      (if (= n 0)
        (begin (display "done") (newline))
        (loop (- n 1)))))
  (loop 100000))
