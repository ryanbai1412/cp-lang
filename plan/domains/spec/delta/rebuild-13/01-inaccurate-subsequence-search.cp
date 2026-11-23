// Codeforces 1955D Inaccurate Subsequence Search
// in: t; per test n m k, then a (n ints), b (m ints); out: per test, the number of good length-m windows of a
@!multitest
let n: 1..=2e5 = read()
let m: 1..=n = read()
let k: 1..=m = read()
let a = read<[n]1..=1e6>()
let b = read<[m]1..=1e6>()

var need: Counter<Int> = Counter(0)
for x in b: need[x] += 1
var have: Counter<Int> = Counter(0)
var matched = 0
var good = 0
for i in a.keys:
    have[a[i]] += 1
    if have[a[i]] <= need[a[i]]: matched += 1
    if i >= m:
        let out = a[i - m]
        if have[out] <= need[out]: matched -= 1
        have[out] -= 1
    if i >= m - 1 and matched >= k: good += 1
print(good)
