// expect: ok, output "0 1 3"
let s = Sieve(100)
print(s.factor(1).len, s.factor(97).len, s.factor(60).len)
