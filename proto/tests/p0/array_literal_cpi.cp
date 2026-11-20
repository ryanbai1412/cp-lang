domain D = 0..3
let primes: [D]Int = [2, 3, 5]
print(primes[D(0)], primes[D(1)], primes[D(2)], sum(primes[i] for i in D))
