// Codeforces 2261 D1 (no restrictions): pair position i with i + ceil(n / 2).
@!multitest
let (n, m): (1..=2e5, 0..=0) = read()
if n == 1:
    print(0)
    return
let half = (n + 1) / 2
print(half)
for i in 0..half: print(i + 1, i + half)
