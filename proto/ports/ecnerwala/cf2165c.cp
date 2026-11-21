@!multitest
let (n0, q): (Int, Int) = read()
let a: [0..n0]Int = read()
let top = a.sorted(desc: true)[..min(n0, 30)]
let n = top.len

for _ in 0..q:
    var buf = top
    let c: Int = read()
    var ans = 0
    for b in (0..30).rev:
        let want = Int(c.bit(b))
        let cnt = count(x.bit(b) for x in buf)
        if cnt > want: break
        if cnt < want:
            let mask = (1 << b) - 1
            let best_i = (0..n).max(by: i => buf[i] & mask)!
            let add = (1 << b) - (buf[best_i] & mask)
            ans += add
            buf[best_i] += add
    print(ans)
