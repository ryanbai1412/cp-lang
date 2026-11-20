@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var freq: [0..=n]Int = 0
for x in a: freq[x] += 1
var ans = 0
for i in 0..=n:
    ans += if freq[i] < i: freq[i] else: freq[i] - i
print(ans)
