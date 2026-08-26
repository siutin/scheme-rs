;; Prime number theory — exercises filter, named let, do loops, assert,
;; number->string with radix, string->number, and/or, car/cdr compositions.

(begin
  (print "prime number theory example")

  ;; --- Prime test using trial division ---

  (define (prime? n)
    (and (> n 1)
         (or (= n 2)
             (and (odd? n)
                  (let loop ((d 3))
                    (cond
                      ((> (* d d) n) #t)
                      ((= 0 (modulo n d)) #f)
                      (else (loop (+ d 2)))))))))

  ;; --- Sieve of Eratosthenes using filter ---

  (define (sieve n)
    (let loop ((nums (list 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20))
               (primes (quote ())))
      (if (null? nums)
        (reverse primes)
        (let ((p (car nums)))
          (loop
            (filter (lambda (x) (not (= 0 (modulo x p)))) (cdr nums))
            (cons p primes))))))

  ;; --- Goldbach conjecture check (even = sum of two primes) ---

  (define (goldbach? n)
    (and (even? n) (> n 2)
         (let loop ((p 2))
           (cond
             ((> p n) #f)
             ((and (prime? p) (prime? (- n p)))
              (list p (- n p)))
             (else (loop (+ p 1)))))))

  ;; --- Display primes up to 20 ---

  (display "primes up to 20: ")
  (display (sieve 20))
  (newline)

  ;; --- Display primes using filter on range ---

  (define all-nums (list 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20))
  (display "primes (filter): ")
  (display (filter prime? all-nums))
  (newline)

  ;; --- Goldbach pairs ---

  (display "goldbach(10) = ")
  (display (goldbach? 10))
  (newline)

  (display "goldbach(20) = ")
  (display (goldbach? 20))
  (newline)

  ;; --- Binary representation using number->string ---

  (display "primes in binary: ")
  (for-each
    (lambda (p)
      (display p)
      (display " = ")
      (display (number->string p 2))
      (display "  "))
    (filter prime? all-nums))
  (newline)

  ;; --- Hex representation ---

  (display "primes in hex: ")
  (for-each
    (lambda (p)
      (display (number->string p 16))
      (display " "))
    (filter prime? all-nums))
  (newline)

  ;; --- Sum of primes using do loop ---

  (define primes (filter prime? all-nums))
  (display "sum of primes: ")
  (display
    (do ((lst primes (cdr lst))
         (sum 0 (+ sum (car lst))))
        ((null? lst) sum)))
  (newline)

  ;; --- car/cdr compositions on prime list ---

  (display "first prime (car): ")
  (display (car primes))
  (newline)

  (display "second prime (cadr): ")
  (display (cadr primes))
  (newline)

  (display "third prime (caddr): ")
  (display (caddr primes))
  (newline)

  (display "rest after first (cdr): ")
  (display (cdr primes))
  (newline)

  (display "rest after second (cddr): ")
  (display (cddr primes))
  (newline)

  ;; --- Assertions ---

  (assert (prime? 2))
  (assert (prime? 13))
  (assert (not (prime? 1)))
  (assert (not (prime? 4)))
  (assert (= 8 (length primes)))
  (assert (= 3 (car (goldbach? 10))))
  (assert (= 3 (car (goldbach? 20))))
  (print "all assertions passed")
)
