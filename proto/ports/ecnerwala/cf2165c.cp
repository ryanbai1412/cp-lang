@!multitest
let (n0, q): (Int, Int) = read()
var a: List<Int> = [read() for _ in 0..n0]
sort(mut a)
let b_bits = 30
let top = if a.len > b_bits: a[(a.len - b_bits)..] else: a
let n = top.len

for _ in 0..q:
    var buf = top
    let c: Int = read()
    var ans = 0
    for b in (0..b_bits).rev:
        let want = Int(c.bit(b))
        var cnt = 0
        for i in 0..n:
            if buf[i].bit(b): cnt += 1
        if cnt > want: break
        if cnt < want:
            var best_v = -1
            var best_i = 0
            let mask = (1 << b) - 1
            for i in 0..n:
                let cnd = buf[i] & mask
                if cnd > best_v:
                    best_v = cnd
                    best_i = i
            let add = (1 << b) - best_v
            ans += add
            buf[best_i] += add
    print(ans)
