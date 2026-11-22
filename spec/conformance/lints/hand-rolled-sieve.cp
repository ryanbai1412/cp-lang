// expect: L0518 at 3
let n = 20
var is_prime: [0..=20]Bool = true
for i in 2..=n:
    if is_prime[i]:
        var j = i * i
        while j <= n:
            is_prime[j] = false
            j += i
print(is_prime[7])
