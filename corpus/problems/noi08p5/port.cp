// Runnable today/ port of NOI '08 P5 (Clouds) for real Rust `cpl build`.
// Stdin: q, m, then q operations (1 t c l r d | 2 t l r | 3 t c). Prints each type-2 answer.
// original: corpus/submissions/6067573.cpp

fn main_solve():
    let q: Int = read()
    let m: Int = read()
    let p = 2 * m
    let lim = 2 * p

    fn emod(a: Int, b: Int) -> Int:
        let r = a % b
        return if r < 0: r + b else: r

    let stride = lim + 3
    var bit0 = [0..stride * stride] _ => 0
    var bit1 = [0..stride * stride] _ => 0

    fn b_add(which: Int, x0: Int, y0: Int, v: Int):
        var x = x0 + 1
        while x <= lim:
            var y = y0 + 1
            while y <= lim:
                let idx = x * stride + y
                if which == 0: bit0[idx] += v
                else: bit1[idx] += v
                y += y & -y
            x += x & -x

    fn b_pref(which: Int, x0: Int, y0: Int) -> Int:
        var ans = 0
        var x = x0
        while x > 0:
            var y = y0
            while y > 0:
                let idx = x * stride + y
                ans += if which == 0: bit0[idx] else: bit1[idx]
                y -= y & -y
            x -= x & -x
        return ans

    fn q_clamped(which: Int, xl: Int, xr: Int, yl: Int, yr: Int) -> Int:
        if xr < 0 or yr < 0 or xl >= lim or yl >= lim: return 0
        let a = max(0, xl)
        let b = min(lim - 1, xr)
        let c = max(0, yl)
        let d = min(lim - 1, yr)
        if a > b or c > d: return 0
        return b_pref(which, b + 1, d + 1) - b_pref(which, a, d + 1) - b_pref(which, b + 1, c) + b_pref(which, a, c)

    fn ds_query(which: Int, xl: Int, xr: Int, yl: Int, yr: Int) -> Int:
        return q_clamped(which, xl, xr, yl, yr) + q_clamped(which, xl - p, xr - p, yl - p, yr - p)

    var c_id: List<Int> = []
    var c_lpos: List<Int> = []
    var c_len: List<Int> = []

    for _ in 0..q:
        let op: Int = read()
        if op == 1:
            let t: Int = read()
            let c: Int = read()
            let l: Int = read()
            let r: Int = read()
            let d: Int = read()
            let clen = r - l
            let raw_pos = if d == -1: p - l else: l
            let l_pos = emod(raw_pos - t, p)
            b_add(0, l_pos, l_pos + clen, 1)
            b_add(1, l_pos + p, l_pos - clen + p, 1)
            c_id.push(c)
            c_lpos.push(l_pos)
            c_len.push(clen)
        elif op == 2:
            let t: Int = read()
            let l: Int = read()
            let r: Int = read()
            let base = emod(-t, p)
            let a1 = ds_query(0, base, base + r, base + l, 6 * m - 1)
            let b2 = base + p
            let a2 = ds_query(1, b2 + max(r + 1, p - r), b2 + p - 1, 0, b2 + p - l)
            print(a1 + a2)
        else:
            let _t: Int = read()
            let c: Int = read()
            for i in 0..c_id.len:
                if c_id[i] == c:
                    let lp = c_lpos[i]
                    let cl = c_len[i]
                    b_add(0, lp, lp + cl, -1)
                    b_add(1, lp + p, lp - cl + p, -1)
                    c_id[i] = -1
                    break

main_solve()
