use std.strings.{is_palindrome}

@!multitest
let (n, k0): (Int, Int) = read()
let a: [0..n]Int = read()
let k = k0 - 1
if k == 0:
    print("YES")
    return

let thresh = a.sorted()[k - 1]
var freq: List<Int> = []
var vals: List<Int> = []
var cur_f = 0
for v in a:
    if v < thresh:
        freq.push(cur_f)
        cur_f = 0
        vals.push(v)
    elif v == thresh:
        cur_f += 1
freq.push(cur_f)

let is_pal = is_palindrome(vals)
let mid = freq.len / 2
let cnt = vals.len + (if freq.len % 2 == 1: freq[mid] else: 0) + 2 * sum(
    min(freq[i], freq[freq.len - 1 - i]) for i in 0..mid
)
print(if is_pal and cnt >= k: "YES" else: "NO")
