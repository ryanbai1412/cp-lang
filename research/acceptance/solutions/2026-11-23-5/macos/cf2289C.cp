@!multitest
let n: 1..=2e5 = read()
let p: [0..n]1..=n = read()
let l = p.prev_smaller()
let r = p.next_smaller()
var ans = 0
for i in 0..n:
    let left = i - (if let j = l[i]: j + 1 else: 0)
    let right = (r[i] ?? n) - 1 - i
    ans += if left == 0 or right == 0: left + right + 1 else: 2
print(ans)
