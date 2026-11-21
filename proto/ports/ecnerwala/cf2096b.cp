@! multitest
let n: Int = read()
let k: Int = read()
let l: [0..n]Int = read()
let r: [0..n]Int = read()
var ans = 0
var mins: List<Int> = []
for i in 0..n:
    ans += max(l[i], r[i])
    mins.push(min(l[i], r[i]))
mins.sort()
for i in n - (k - 1)..n:
    ans += mins[i]
print(ans + 1)
