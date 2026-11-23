// A recurrence over the previous entry that is not `i!` is not a factorial table.
let n: 1..=1000000 = read()
var pw: List<Int> = [1 for _ in 0..=n]
for i in 1..=n:
    pw[i] = pw[i - 1] * 2 % 998244353
var ways: List<Int> = [1 for _ in 0..=n]
for i in 1..=n:
    ways[i] = ways[i - 1] * (i + 1) % 998244353
print(pw[n], ways[n])
