;; List operations — map, filter, reduce
(begin
  (print "list operations example")

  ;; map: apply f to each element
  (define map
    (lambda (f lst)
      (if (null? lst)
        (quote ())
        (cons (f (car lst)) (map f (cdr lst))))))

  ;; filter: keep elements where pred is true
  (define filter
    (lambda (pred lst)
      (if (null? lst)
        (quote ())
        (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst))))))

  ;; reduce: fold left
  (define reduce
    (lambda (f init lst)
      (if (null? lst)
        init
        (reduce f (f init (car lst)) (cdr lst)))))

  (define nums (list 1 2 3 4 5 6 7 8 9 10))

  ;; sum via reduce
  (display "sum 1..10: ")
  (display (reduce + 0 nums))
  (newline)

  ;; squares via map
  (display "squares: ")
  (display (map (lambda (x) (* x x)) nums))
  (newline)

  ;; evens via filter
  (display "evens: ")
  (display (filter even? nums))
  (newline))
