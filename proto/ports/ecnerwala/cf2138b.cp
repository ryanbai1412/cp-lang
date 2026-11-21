use std.seq.{next_less, prev_greater, suffix_scan}

@!multitest
let (n, q): (Int, Int) = read()
domain I = 1..=n
let a: [I]Int = read()

let prev_gt = a.prev_greater(strict: false)
let next_lt = a.next_less(strict: false)
var bad: [I]Int = n + 1
for i in I:
    if let l = prev_gt[i], let r = next_lt[i]:
        bad[l] min= r.to_idx + 1
let min_bad = suffix_scan(bad, min)

for _ in 0..q:
    let (l, r): (I, Int) = read()
    print(if r >= min_bad[l]: "NO" else: "YES")
