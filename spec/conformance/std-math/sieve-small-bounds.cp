// expect: ok, output "NO NO YES NO"
let s = Sieve(2)
print(yes(s.is_prime(0)), yes(s.is_prime(1)), yes(s.is_prime(2)), yes(Sieve(0).is_prime(0)))
