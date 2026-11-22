let n = 20
var is_prime = List.filled(n + 1, true)
for i in 2..=n:
    if is_prime[i]:
        var j = i * i
        while j <= n:
            is_prime[j] = false
            j += i
print(is_prime[7])
