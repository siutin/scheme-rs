;; Closures with set! — counter and accumulator
(begin
  (print "closures example")

  ;; make-counter: returns a function that increments and returns count
  (define make-counter
    (lambda ()
      (let ((count 0))
        (lambda ()
          (set! count (+ count 1))
          count))))

  (define c1 (make-counter))
  (define c2 (make-counter))

  (display "counter 1: ")
  (display (c1))
  (display " ")
  (display (c1))
  (display " ")
  (display (c1))
  (newline)

  (display "counter 2: ")
  (display (c2))
  (newline)

  ;; make-accumulator: returns a function that adds to a running sum
  (define make-accumulator
    (lambda (init)
      (let ((total init))
        (lambda (n)
          (set! total (+ total n))
          total))))

  (define acc (make-accumulator 100))
  (display "accumulator: ")
  (display (acc 10))
  (display " ")
  (display (acc 25))
  (display " ")
  (display (acc -5))
  (newline))
