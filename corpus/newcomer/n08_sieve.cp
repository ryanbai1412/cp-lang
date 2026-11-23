let n: 1..=1000000 = read()
var comp: [0..=n]Bool = false
var primes: List<Int> = []
for i in 2..=n:
    if not comp[i]:
        primes.push(i)
        var j = i * i
        while j <= n:  // expect: hand_rolled_sieve
            comp[j] = true
            j += i
print(primes.len)
