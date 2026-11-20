@!multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()
var cnt_a: [0..=n]Int = 0
var cnt_chk: [0..=n]Int = 0
for i in 0..n:
    cnt_a[i + 1] = cnt_a[i] + a[i]
    cnt_chk[i + 1] = cnt_chk[i] + (a[i] ^ (i & 1))
for _ in 0..q:
    let (l1, r): (Int, Int) = read()
    let l = l1 - 1
    if (r - l) % 3 != 0 or (cnt_a[r] - cnt_a[l]) % 3 != 0:
        print(-1)
    else:
        let raw = cnt_chk[r] - cnt_chk[l]
        let v = min(raw, (r - l) - raw)
        print((r - l) / 3 + Int(v == 0))
