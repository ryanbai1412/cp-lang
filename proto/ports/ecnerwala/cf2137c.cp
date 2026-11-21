// Codeforces 2137 C: maximize a + b after transferring a divisor of b into a
// so the sum is even.
@!multitest
let (a, b): (Int, Int) = read()
let ans = if a % 2 == 1 and b % 2 == 1: a * b + 1
elif b % 2 == 0 and (a * b) % 4 == 0: a * b / 2 + 2
else: -1
print(ans)
