@!multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()
let cnt_a = prefix(a)
let cnt_chk = prefix([a[i] ^ (i & 1) for i in 0..n])

for _ in 0..q:
    let (l1, r): (Int, Int) = read()
    let l = l1 - 1
    if (r - l) % 3 != 0 or cnt_a.sum(l..r) % 3 != 0:
        print(-1)
    else:
        let raw = cnt_chk.sum(l..r)
        print((r - l) / 3 + Int(min(raw, r - l - raw) == 0))
