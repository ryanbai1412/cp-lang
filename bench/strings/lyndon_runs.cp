
let n = 100_000
var s = List.filled(n, 0)
var x = 246813579
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = (x >> 16) & 3
let runs = lyndon_runs(s)
var ans = runs.len
for run in runs:
    ans += run.l * 3 + run.r * 5 + run.period * 7
print(ans)
