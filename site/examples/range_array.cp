let n: 1..=100 = read()
let k: 1..=100 = read()
let a: [0..n]0..=k = read()

var cnt: [0..=k]Int = 0
for i in 0..n:
    cnt[a[i]] += 1
print(cnt)
