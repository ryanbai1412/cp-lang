// What is above `@!multitest` runs once: the sieve is built before the test count is read, then shared.
let LIM = 100
domain N = 0..=LIM
var composite: [N]Bool
var built = 0
for i in N(2)..:
    if not composite[i]:
        var j = i.ord * i.ord
        while j <= LIM:
            composite[N(j)] = true
            j += i.ord
built += 1

@!multitest
let n: 2..=LIM = read()
var primes = 0
for i in N(2)..=N(n):
    if not composite[i]: primes += 1
print(primes, built)
