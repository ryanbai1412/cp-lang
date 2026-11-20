//! Fast polynomial arithmetic and formal power series over `998244353`
//! (with 3-prime NTT convolution for arbitrary moduli).
//!
//! Polynomials are lists of coefficients, lowest degree first (`p[k]` is
//! the coefficient of `x^k`). Power-series helpers take `keep`, the number
//! of coefficients of the answer, so `exp_trunc(log_trunc(p, n), n)` is `p`
//! truncated to `n` terms.

use std.mod.{Mod}

/// The standard NTT prime `998244353 = 119 * 2^23 + 1`, with primitive root `3`.
pub type Z = Mod<998244353>

// The transforms work in Montgomery form with radix 2^32 on the three NTT primes
// P1 = 998244353 (2^23), P2 = 469762049 (2^26), P3 = 167772161 (2^25), all below
// 2^30 with primitive root 3. Values stay in `[0, 4P)` between butterflies, so
// every product is below 2^62 and reductions are branch-free.

// Montgomery helpers (R = 2^32) for primes p < 2^30.
fn mred(x: Int, p: Int, pinv: Int) -> Int =
    (x + (((x & 4294967295) * pinv) & 4294967295) * p) >> 32

fn mred1(x: Int, p: Int, pinv: Int) -> Int:
    let r = mred(x, p, pinv)
    if r >= p: r - p else: r

fn red2(x: Int, p2: Int) -> Int = x + ((x >> 63) & p2)

fn red_p(x: Int, p: Int, p2: Int) -> Int:
    let v = red2(x - p2, p2) - p
    v + ((v >> 63) & p)

fn add_p(a: Int, b: Int, p: Int) -> Int:
    let s = a + b
    if s >= p: s - p else: s

fn sub_p(a: Int, b: Int, p: Int) -> Int:
    let s = a - b
    if s < 0: s + p else: s

fn add7(a: Int, b: Int) -> Int:
    let s = a + b
    if s >= 7: s - 7 else: s

fn pow_mod_i(base: Int, exp: Int, m: Int) -> Int:
    var res = 1 % m
    var b = base % m
    var e = exp
    while e > 0:
        if (e & 1) == 1:
            res = res * b % m
        b = b * b % m
        e = e >> 1
    res

fn build_facts(n: Int) -> (List<Int>, List<Int>):
    let p = Z.MOD
    var f = List.filled(n + 1, 1)
    var fi = List.filled(n + 1, 1)
    var prod = 1
    for i in 1..=n:
        prod = prod * i % p
        f[i] = prod
    if n > 0:
        fi[n] = pow_mod_i(f[n], p - 2, p)
        var i = n
        while i >= 1:
            fi[i - 1] = fi[i] * i % p
            i -= 1
    (f, fi)

fn next_pow2(n: Int) -> Int:
    var s = 1
    while s < n:
        s = s << 1
    s

fn ilog2_pow2(size: Int) -> Int:
    var h = 0
    while (1 << h) < size:
        h += 1
    h

struct NttRoots:
    p: Int
    pinv: Int
    one_m: Int
    rt: List<Int>
    irt: List<Int>
    r3: List<Int>
    ir3: List<Int>
    sw: List<Int>
    isw: List<Int>

fn build_roots(p: Int) -> NttRoots:
    let pinv = if p == 998244353: 998244351 elif p == 469762049: 469762047 else: 167772159
    let one_m = if p == 998244353: 301989884 elif p == 469762049: 67108855 else: 100663271
    let max_k = ctz(p - 1)
    var rt = List.filled(max_k + 1, 0)
    var irt = List.filled(max_k + 1, 0)
    let rmax = pow_mod_i(3, (p - 1) >> max_k, p)
    rt[max_k] = rmax * one_m % p
    irt[max_k] = pow_mod_i(rmax, p - 2, p) * one_m % p
    var k = max_k
    while k > 0:
        rt[k - 1] = mred1(rt[k] * rt[k], p, pinv)
        irt[k - 1] = mred1(irt[k] * irt[k], p, pinv)
        k -= 1
    let nr = max_k - 2
    var r3 = List.filled(nr, 0)
    var ir3 = List.filled(nr, 0)
    var sw = List.filled(nr + 1, 0)
    var isw = List.filled(nr + 1, 0)
    var ip = one_m
    var fp = one_m
    for j in 0..nr:
        r3[j] = mred1(rt[j + 3] * ip, p, pinv)
        ir3[j] = mred1(irt[j + 3] * fp, p, pinv)
        ip = mred1(ip * irt[j + 3], p, pinv)
        fp = mred1(fp * rt[j + 3], p, pinv)
    sw[0] = rt[2]
    isw[0] = irt[2]
    for j in 0..nr:
        sw[j + 1] = mred1(r3[j] * irt[2], p, pinv)
        isw[j + 1] = mred1(ir3[j] * rt[2], p, pinv)
    NttRoots(p, pinv, one_m, rt, irt, r3, ir3, sw, isw)

fn pow_bits(s_in: Int, shift: Int, one_m: Int, p: Int, pinv: Int, tab: List<Int>) -> Int:
    var r = one_m
    var s = s_in
    var k = shift
    while s > 0:
        if (s & 1) == 1:
            r = mred1(r * tab[k], p, pinv)
        s = s >> 1
        k += 1
    r

fn dif_stages<D>(
    mut d: [D]Int,
    base: Int,
    sub_h: Int,
    len_start: Int,
    len_end: Int,
    c_idx: Int,
    roots: NttRoots
):
    let p = roots.p
    let pinv = roots.pinv
    let one_m = roots.one_m
    let imag_m = roots.rt[2]
    let p2 = p + p
    var slen = len_start
    while slen < len_end:
        let step = 1 << (sub_h - slen - 2)
        let blocks = 1 << slen
        var rot = if c_idx == 0: one_m else: pow_bits(c_idx << slen, 3, one_m, p, pinv, roots.rt)
        for s in 0..blocks:
            let rot2 = mred1(rot * rot, p, pinv)
            let rot3 = mred1(rot2 * rot, p, pinv)
            let off = base + (s << (sub_h - slen))
            let o1 = off + step
            let o2 = o1 + step
            let o3 = o2 + step
            for i in 0..step:
                let a0 = red2(d[off + i] - p2, p2)
                let a1 = mred(d[o1 + i] * rot, p, pinv)
                let a2 = mred(d[o2 + i] * rot2, p, pinv)
                let a3 = mred(d[o3 + i] * rot3, p, pinv)
                let a13i = mred((a1 + p2 - a3) * imag_m, p, pinv)
                let t02 = red2(a0 + a2 - p2, p2)
                let t13 = red2(a1 + a3 - p2, p2)
                let d02 = red2(a0 - a2, p2)
                d[off + i] = t02 + t13
                d[o1 + i] = t02 + p2 - t13
                d[o2 + i] = d02 + a13i
                d[o3 + i] = d02 + p2 - a13i
            if s + 1 < blocks:
                rot = mred1(rot * roots.r3[ctz(s ^ -1)], p, pinv)
        slen += 2

fn dit_stages<D>(
    mut d: [D]Int,
    base: Int,
    sub_h: Int,
    len_start: Int,
    len_end: Int,
    c_idx: Int,
    roots: NttRoots
):
    let p = roots.p
    let pinv = roots.pinv
    let one_m = roots.one_m
    let iimag_m = roots.irt[2]
    let p2 = p + p
    var slen = len_start
    while slen >= len_end:
        let step = 1 << (sub_h - slen)
        let blocks = 1 << (slen - 2)
        var irot = if c_idx == 0: one_m else: pow_bits(c_idx << (slen - 2), 3, one_m, p, pinv, roots.irt)
        for s in 0..blocks:
            let irot2 = mred1(irot * irot, p, pinv)
            let irot3 = mred1(irot2 * irot, p, pinv)
            let off = base + (s << (sub_h - slen + 2))
            let o1 = off + step
            let o2 = o1 + step
            let o3 = o2 + step
            for i in 0..step:
                let a0 = d[off + i]
                let a1 = d[o1 + i]
                let a2 = d[o2 + i]
                let a3 = d[o3 + i]
                let a23i = mred((a2 + p2 - a3) * iimag_m, p, pinv)
                let t01 = red2(a0 + a1 - p2, p2)
                let t23 = red2(a2 + a3 - p2, p2)
                let d01 = red2(a0 - a1, p2)
                d[off + i] = red2(t01 + t23 - p2, p2)
                d[o1 + i] = mred((d01 + a23i) * irot, p, pinv)
                d[o2 + i] = mred((t01 + p2 - t23) * irot2, p, pinv)
                d[o3 + i] = mred((d01 + p2 - a23i) * irot3, p, pinv)
            if s + 1 < blocks:
                irot = mred1(irot * roots.ir3[ctz(s ^ -1)], p, pinv)
        slen -= 2

// In-place radix-4 Gentleman-Sande DIF (`op == 0`) and Cooley-Tukey DIT (`op == 1` or `op == 2`
// for R^2-scaled Montgomery pointwise products) on `a[0..size]`.
fn ntt_core_r(mut a: List<Int>, size: Int, op: Int, roots: NttRoots):
    if size <= 1:
        return
    let p = roots.p
    let pinv = roots.pinv
    let one_m = roots.one_m
    let p2 = p + p
    let h = ilog2_pow2(size)
    var buf: [0..size]Int
    for i in 0..size: buf[i] = a[i]
    if op == 0:
        var len = 0
        if (h & 1) == 1:
            let step = size >> 1
            for i in 0..step:
                let l = buf[i]
                let r = buf[i + step]
                buf[i] = l + r
                buf[i + step] = l + p2 - r
            len = 1
        dif_stages(mut buf, 0, h, len, h, 0, roots)
        for i in 0..size:
            a[i] = red_p(buf[i], p, p2)
        return
    dit_stages(mut buf, 0, h, h, 2, 0, roots)
    let scale_base = if op == 2: pow_mod_i(size, p - 2, p) * ((one_m * one_m) % p) % p else: pow_mod_i(size, p - 2, p) * one_m % p
    if (h & 1) == 1:
        let step = size >> 1
        for i in 0..step:
            let l = buf[i]
            let r = buf[i + step]
            a[i] = mred1((l + r) * scale_base, p, pinv)
            a[i + step] = mred1((l + p2 - r) * scale_base, p, pinv)
    else:
        for i in 0..size:
            a[i] = mred1(buf[i] * scale_base, p, pinv)

fn ntt_core(mut a: List<Int>, size: Int, op: Int, p: Int):
    ntt_core_r(mut a, size, op, build_roots(p))

// Fused size-4 polynomial product `A(x) * B(x) mod (x^4 - w)` for two adjacent blocks
// `(o1, o1 + 4)` with roots `+w` and `-w` (in Montgomery form).
fn mul4_pair<D>(mut d: [D]Int, o1: Int, o2: Int, w: Int, p: Int, pinv: Int, sq: Bool):
    let p2 = p + p
    for sgn in 0..2:
        let oa = o1 + (sgn << 2)
        let ws = if sgn == 0: w else: p - w
        let a0 = red_p(d[oa], p, p2)
        let a1 = red_p(d[oa + 1], p, p2)
        let a2 = red_p(d[oa + 2], p, p2)
        let a3 = red_p(d[oa + 3], p, p2)
        var u0 = 0
        var v0 = 0
        var u1 = 0
        var v1 = 0
        var u2 = 0
        var v2 = 0
        var u3 = 0
        if sq:
            u0 = a0 * a0
            v0 = ((a1 * a3) << 1) + a2 * a2
            u1 = (a0 * a1) << 1
            v1 = (a2 * a3) << 1
            u2 = ((a0 * a2) << 1) + a1 * a1
            v2 = a3 * a3
            u3 = (a0 * a3 + a1 * a2) << 1
        else:
            let ob = o2 + (sgn << 2)
            let b0 = red_p(d[ob], p, p2)
            let b1 = red_p(d[ob + 1], p, p2)
            let b2 = red_p(d[ob + 2], p, p2)
            let b3 = red_p(d[ob + 3], p, p2)
            u0 = a0 * b0
            v0 = a1 * b3 + a2 * b2 + a3 * b1
            u1 = a0 * b1 + a1 * b0
            v1 = a2 * b3 + a3 * b2
            u2 = a0 * b2 + a1 * b1 + a2 * b0
            v2 = a3 * b3
            u3 = a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0
        d[oa] = red2(mred(u0, p, pinv) + mred(mred(v0, p, pinv) * ws, p, pinv) - p2, p2)
        d[oa + 1] = red2(mred(u1, p, pinv) + mred(mred(v1, p, pinv) * ws, p, pinv) - p2, p2)
        d[oa + 2] = red2(mred(u2, p, pinv) + mred(mred(v2, p, pinv) * ws, p, pinv) - p2, p2)
        d[oa + 3] = mred(u3, p, pinv)

// Fused size-4 autocorrelation for a pair of partner blocks `(oa, ob)` with roots
// `w` (for `oa`) and `winv` (for `ob`), both in Montgomery form.
fn autocorr4_blk<D>(mut d: [D]Int, oa: Int, ob: Int, w: Int, winv: Int, p: Int, pinv: Int):
    let p2 = p + p
    let a0 = red_p(d[oa], p, p2)
    let a1 = red_p(d[oa + 1], p, p2)
    let a2 = red_p(d[oa + 2], p, p2)
    let a3 = red_p(d[oa + 3], p, p2)
    let b0 = red_p(d[ob], p, p2)
    let b1 = red_p(d[ob + 1], p, p2)
    let b2 = red_p(d[ob + 2], p, p2)
    let b3 = red_p(d[ob + 3], p, p2)
    let c0 = mred(a0 * b0 + a1 * b1 + a2 * b2 + a3 * b3, p, pinv)
    let u1r = mred(a1 * b0 + a2 * b1 + a3 * b2, p, pinv)
    let v1r = mred(a0 * b3, p, pinv)
    let u2r = mred(a2 * b0 + a3 * b1, p, pinv)
    let v2r = mred(a0 * b2 + a1 * b3, p, pinv)
    let u3r = mred(a3 * b0, p, pinv)
    let v3r = mred(a0 * b1 + a1 * b2 + a2 * b3, p, pinv)
    d[oa] = c0
    d[oa + 1] = red2(u1r + mred(v1r * winv, p, pinv) - p2, p2)
    d[oa + 2] = red2(u2r + mred(v2r * winv, p, pinv) - p2, p2)
    d[oa + 3] = red2(u3r + mred(v3r * winv, p, pinv) - p2, p2)
    if oa != ob:
        d[ob] = c0
        d[ob + 1] = red2(mred(u3r * w, p, pinv) + v3r - p2, p2)
        d[ob + 2] = red2(mred(u2r * w, p, pinv) + v2r - p2, p2)
        d[ob + 3] = red2(mred(u1r * w, p, pinv) + v1r - p2, p2)

fn autocorr4_range<D>(
    mut d: [D]Int,
    b1: Int,
    b2: Int,
    s1_start: Int,
    s2_start: Int,
    pairs2: Int,
    w_idx: Int,
    roots: NttRoots
):
    let p = roots.p
    let pinv = roots.pinv
    let one_m = roots.one_m
    var w = pow_bits(w_idx, 1, one_m, p, pinv, roots.rt)
    var winv = pow_bits(w_idx, 1, one_m, p, pinv, roots.irt)
    for k in 0..pairs2:
        let s1 = s1_start + (k << 1)
        let s2 = s2_start - (k << 1)
        autocorr4_blk(mut d, b1 + (s1 << 2), b2 + (s2 << 2), w, winv, p, pinv)
        autocorr4_blk(mut d, b1 + ((s1 + 1) << 2), b2 + ((s2 - 1) << 2), p - w, p - winv, p, pinv)
        if k + 1 < pairs2:
            let t = ctz(k ^ -1)
            w = mred1(w * roots.sw[t], p, pinv)
            winv = mred1(winv * roots.isw[t], p, pinv)

// Cache-blocked pure radix-4 convolution on an even-height (`h2`) buffer `d` of size
// `sides * (1 << h2)`. `half_id == 0` computes mod `(x^{2^h2} - 1)`, `half_id == 1`
// computes mod `(x^{2^h2} + 1)`.
fn conv_exec_even<D>(mut d: [D]Int, n_buf: Int, h2: Int, mode: Int, half_id: Int, roots: NttRoots):
    let p = roots.p
    let pinv = roots.pinv
    let one_m = roots.one_m
    let imag_m = roots.rt[2]
    let iimag_m = roots.irt[2]
    let sides = if mode == 0: 2 else: 1
    let split_len = if h2 <= 12: 0 else: h2 - 12
    let sub_h = h2 - split_len
    let num_chunks = 1 << split_len
    let c_off = half_id << split_len
    let blocks4 = 1 << (sub_h - 2)
    for side in 0..sides:
        dif_stages(mut d, side * n_buf, h2, 0, split_len, half_id, roots)
    if mode != 2:
        let sq = mode == 1
        let half_b = blocks4 >> 1
        for lc in 0..num_chunks:
            let c = c_off + lc
            let c0 = lc << sub_h
            for side in 0..sides:
                dif_stages(mut d, side * n_buf + c0, sub_h, 0, sub_h - 2, c, roots)
            var w = if c == 0: one_m else: pow_bits(c << (sub_h - 2), 1, one_m, p, pinv, roots.rt)
            for k in 0..half_b:
                let off = c0 + (k << 3)
                mul4_pair(mut d, off, n_buf + off, w, p, pinv, sq)
                if k + 1 < half_b:
                    w = mred1(w * roots.sw[ctz(k ^ -1)], p, pinv)
            dit_stages(mut d, c0, sub_h, sub_h - 2, 2, c, roots)
    else:
        var big_b = if half_id == 0: 0 else: num_chunks
        let big_end = if half_id == 0: num_chunks else: num_chunks + 1
        while big_b < big_end:
            let pairs = if big_b <= 1: 1 else: big_b >> 1
            for off_c in 0..pairs:
                let c1 = if big_b == 0: 0 else: big_b + off_c
                let c2 = if big_b == 0: 0 else: 2 * big_b - 1 - off_c
                let lc1 = c1 - c_off
                let lc2 = c2 - c_off
                let count = if lc1 == lc2: 1 else: 2
                for side in 0..count:
                    let c = if side == 0: c1 else: c2
                    let lc = if side == 0: lc1 else: lc2
                    dif_stages(mut d, lc << sub_h, sub_h, 0, sub_h - 2, c, roots)
                let base1 = lc1 << sub_h
                if big_b == 1:
                    autocorr4_range(mut d, base1, base1, 0, blocks4 - 1, blocks4 >> 2, 1 << (sub_h - 2), roots)
                elif lc1 == lc2:
                    autocorr4_blk(mut d, base1, base1, one_m, one_m, p, pinv)
                    autocorr4_blk(mut d, base1 + 4, base1 + 4, p - one_m, p - one_m, p, pinv)
                    if blocks4 > 2:
                        autocorr4_blk(mut d, base1 + 8, base1 + 12, imag_m, iimag_m, p, pinv)
                    var blk = 4
                    while blk < blocks4:
                        autocorr4_range(mut d, base1, base1, blk, (blk << 1) - 1, blk >> 2, blk, roots)
                        blk = blk << 1
                else:
                    autocorr4_range(mut d, base1, lc2 << sub_h, 0, blocks4 - 1, blocks4 >> 1, c1 << (sub_h - 2), roots)
                for side in 0..count:
                    let c = if side == 0: c1 else: c2
                    let lc = if side == 0: lc1 else: lc2
                    dit_stages(mut d, lc << sub_h, sub_h, sub_h - 2, 2, c, roots)
            big_b = if big_b == 0: 1 else: big_b << 1
    dit_stages(mut d, 0, h2, split_len, 2, half_id, roots)

// Mixed-radix `7 * 2^{h2}` autocorrelation mod 998244353 (`998244353 - 1 = 7 * 17 * 2^23`).
// Conjugate symmetry `C_{7-r}(y) = C_r(y^{-1})` cuts the 7 sub-problems to 11 size-`M`
// transforms on a single `2 * M` buffer, with `na <= 3.5 * M` (at most 4 chunks).
fn load_odd_i<D>(mut d: [D]Int, base: Int, src: List<Int>, half: Int, p2: Int, neg: Bool):
    let n = src.len
    for i in 0..half:
        let l = if i < n: src[i] else: 0
        let r = if i + half < n: src[i + half] else: 0
        d[base + i] = if neg: l + p2 - r else: l + r

fn load_odd_z<D>(mut d: [D]Int, base: Int, src: List<Z>, half: Int, p2: Int, neg: Bool):
    let n = src.len
    for i in 0..half:
        let l = if i < n: src[i].v else: 0
        let r = if i + half < n: src[i + half].v else: 0
        d[base + i] = if neg: l + p2 - r else: l + r

fn conv_core(a: List<Int>, b: List<Int>, out_len: Int, mode: Int, p: Int) -> List<Int>:
    let na = a.len
    let nb = b.len
    if na == 0 or nb == 0:
        return []
    let min_ab = if na < nb: na else: nb
    if min_ab <= 60:
        var res: [0..out_len]Int
        for i in 0..na:
            let ai = a[i]
            if ai != 0:
                let lim = if nb < out_len - i: nb else: out_len - i
                for j in 0..lim:
                    res[i + j] = (res[i + j] + ai * b[j]) % p
        return [res[i] for i in 0..out_len]
    let roots = build_roots(p)
    let size = next_pow2(out_len)
    let h = ilog2_pow2(size)
    let sides = if mode == 0: 2 else: 1
    let pinv = roots.pinv
    let one_m = roots.one_m
    let p2 = p + p
    let scale = pow_mod_i(size >> 2, p - 2, p) * ((one_m * one_m) % p) % p
    var out = List.filled(out_len, 0)
    if (h & 1) == 0:
        var d: [0..sides * size]Int
        for i in 0..na: d[i] = a[i]
        if mode == 0:
            for i in 0..nb: d[size + i] = b[i]
        conv_exec_even(mut d, size, h, mode, 0, roots)
        if mode == 2:
            let sh = na - 1
            out[sh] = mred1(d[0] * scale, p, pinv)
            for k in 1..na:
                let val = mred1(d[k] * scale, p, pinv)
                out[sh + k] = val
                out[sh - k] = val
        else:
            for i in 0..out_len:
                out[i] = mred1(d[i] * scale, p, pinv)
        return out
    let half = size >> 1
    let h2 = h - 1
    let lim0 = min(half, out_len)
    var d: [0..sides * half]Int
    for pass in 0..2:
        let neg = pass == 1
        load_odd_i(mut d, 0, a, half, p2, neg)
        if mode == 0: load_odd_i(mut d, half, b, half, p2, neg)
        conv_exec_even(mut d, half, h2, mode, pass, roots)
        if mode == 2:
            let sh = na - 1
            for k in 0..na:
                let v = mred1(d[k] * scale, p, pinv)
                out[sh + k] = if neg: add_p(out[sh + k], v, p) else: v
        elif not neg:
            for i in 0..lim0: out[i] = mred1(d[i] * scale, p, pinv)
        else:
            for i in 0..lim0:
                let lv = out[i]
                let rv = mred1(d[i] * scale, p, pinv)
                out[i] = add_p(lv, rv, p)
                if i + half < out_len: out[i + half] = sub_p(lv, rv, p)
    if mode == 2:
        let sh = na - 1
        for k in 1..na: out[sh - k] = out[sh + k]
    out

fn conv_z(a: List<Z>, b: List<Z>, mode: Int) -> List<Z>:
    let na = a.len
    let nb = b.len
    if na == 0 or nb == 0:
        return []
    let out_len = na + nb - 1
    let p = Z.MOD
    if min(na, nb) <= 60:
        var res = List.filled(out_len, Z.zero)
        for i in 0..na:
            if a[i] != 0:
                for j in 0..nb: res[i + j] += a[i] * b[j]
        return res
    let pinv = 998244351
    let p2 = 1996488706
    let roots = build_roots(p)
    let size = next_pow2(out_len)
    let h = ilog2_pow2(size)
    let sides = if mode == 0: 2 else: 1
    let scale = pow_mod_i(size >> 2, p - 2, p) * 932051910 % p
    var out = List.filled(out_len, Z.zero)
    if (h & 1) == 0:
        var d: [0..sides * size]Int
        for i in 0..na: d[i] = a[i].v
        if mode == 0:
            for i in 0..nb: d[size + i] = b[i].v
        conv_exec_even(mut d, size, h, mode, 0, roots)
        if mode == 2:
            let sh = na - 1
            for k in 0..na: out[sh + k].v = mred1(d[k] * scale, p, pinv)
            for k in 1..na: out[sh - k] = out[sh + k]
        else:
            for i in 0..out_len: out[i].v = mred1(d[i] * scale, p, pinv)
        return out
    let half = size >> 1
    let h2 = h - 1
    let lim0 = min(half, out_len)
    var d: [0..sides * half]Int
    for pass in 0..2:
        let neg = pass == 1
        load_odd_z(mut d, 0, a, half, p2, neg)
        if mode == 0: load_odd_z(mut d, half, b, half, p2, neg)
        conv_exec_even(mut d, half, h2, mode, pass, roots)
        if mode == 2:
            let sh = na - 1
            for k in 0..na:
                let v = mred1(d[k] * scale, p, pinv)
                out[sh + k].v = if neg: add_p(out[sh + k].v, v, p) else: v
        elif not neg:
            for i in 0..lim0: out[i].v = mred1(d[i] * scale, p, pinv)
        else:
            for i in 0..lim0:
                let lv = out[i].v
                let rv = mred1(d[i] * scale, p, pinv)
                out[i].v = add_p(lv, rv, p)
                if i + half < out_len: out[i + half].v = sub_p(lv, rv, p)
    if mode == 2:
        let sh = na - 1
        for k in 1..na: out[sh - k] = out[sh + k]
    out

fn detect_sym_mode(a: List<Int>, b: List<Int>) -> Int:
    let n = a.len
    if n != b.len: return 0
    var eq = true
    for i in 0..n:
        if a[i] != b[i]:
            eq = false
            break
    if eq: return 1
    var rev = true
    for i in 0..n:
        if a[i] != b[n - 1 - i]:
            rev = false
            break
    if rev: return 2
    0

fn detect_sym_mode_z(a: List<Z>, b: List<Z>) -> Int:
    let n = a.len
    if n != b.len: return 0
    var eq = true
    for i in 0..n:
        if a[i].v != b[i].v:
            eq = false
            break
    if eq: return 1
    var rev = true
    for i in 0..n:
        if a[i].v != b[n - 1 - i].v:
            rev = false
            break
    if rev: return 2
    0

// P1 = 998244353 helpers on raw Int lists (values in [0, P1)).
fn dif_p1(mut a: List<Int>):
    ntt_core(mut a, a.len, 0, 998244353)

fn dit_p1(mut a: List<Int>):
    ntt_core(mut a, a.len, 1, 998244353)

fn dit_p1_r2(mut a: List<Int>):
    ntt_core(mut a, a.len, 2, 998244353)

fn convolve_p1_raw(a: List<Int>, b: List<Int>) -> List<Int>:
    if a.len == 0 or b.len == 0: return []
    conv_core(a, b, a.len + b.len - 1, detect_sym_mode(a, b), 998244353)

/// The NTT of `a` in place (DIF forward, DIT inverse already divided by `a.len`);
/// `a.len` must be a power of two, at most `2^23`.
pub fn ntt(mut a: List<Z>, invert: Bool):
    let n = a.len
    assert n > 0 and (n & (n - 1)) == 0
    if n == 1: return
    var raw = List.filled(n, 0)
    for i in 0..n: raw[i] = a[i].v
    if invert: dit_p1(mut raw)
    else: dif_p1(mut raw)
    for i in 0..n: a[i] = Z(raw[i])

/// Exact polynomial product over `Z = Mod<998244353>`.
pub fn convolve(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.len == 0 or b.len == 0: return []
    conv_z(a, b, detect_sym_mode_z(a, b))

/// Middle product `c[i] = sum_{j=0}^{b.len-1} a[i + j] * b[j]` for
/// `0 <= i < a.len - b.len + 1`. Uses a single circular NTT of size `next_pow2(a.len)`.
pub fn middle_product(a: List<Z>, b: List<Z>) -> List<Z>:
    let na = a.len
    let nb = b.len
    if nb == 0 or na < nb: return []
    let out_len = na - nb + 1
    let p = 998244353
    if nb <= 32 or out_len <= 32:
        var out = List.filled(out_len, Z.zero)
        for i in 0..out_len:
            var acc = 0
            for j in 0..nb:
                acc = (acc + a[i + j].v * b[j].v) % p
            out[i] = Z(acc)
        return out
    let size = next_pow2(na)
    var ra = List.filled(na, 0)
    for i in 0..na: ra[i] = a[i].v
    var rb = List.filled(nb, 0)
    for j in 0..nb: rb[j] = b[nb - 1 - j].v
    let rc = conv_core(ra, rb, size, 0, 998244353)
    var out = List.filled(out_len, Z.zero)
    for i in 0..out_len: out[i] = Z(rc[nb - 1 + i])
    out

fn head(p: List<Z>, n: Int) -> List<Z>:
    let m = min(p.len, n)
    var out = List.filled(m, Z.zero)
    for i in 0..m: out[i] = p[i]
    out

fn pad_to(p: List<Z>, n: Int) -> List<Z>:
    var out = List.filled(n, Z.zero)
    let m = min(p.len, n)
    for i in 0..m: out[i] = p[i]
    out

fn trim_zeros(p: List<Z>) -> List<Z>:
    var n = p.len
    while n > 0 and p[n - 1] == 0: n -= 1
    head(p, n)

fn rev_poly(p: List<Z>) -> List<Z>:
    let n = p.len
    var out = List.filled(n, Z.zero)
    for i in 0..n: out[i] = p[n - 1 - i]
    out

fn add_poly(a: List<Z>, b: List<Z>) -> List<Z>:
    let n = max(a.len, b.len)
    var out = List.filled(n, Z.zero)
    for i in 0..a.len: out[i] = a[i]
    for i in 0..b.len: out[i] += b[i]
    out

/// Formal derivative `p'(x)` over `Z`.
pub fn derivative(p: List<Z>) -> List<Z>:
    if p.len <= 1: return []
    let n = p.len - 1
    var out = List.filled(n, Z.zero)
    for i in 0..n: out[i] = p[i + 1] * Z(i + 1)
    out

/// Formal integral `integral(p)` with constant term `0` over `Z`.
pub fn integral(p: List<Z>) -> List<Z>:
    let n = p.len
    if n == 0: return []
    let (fact, ifact) = build_facts(n)
    let mod_p = 998244353
    var out = List.filled(n + 1, Z.zero)
    for i in 0..n:
        let inv_i = (ifact[i + 1] * fact[i]) % mod_p
        out[i + 1] = Z((p[i].v * inv_i) % mod_p)
    out

fn sub_z(p: List<Z>, lo: Int, hi: Int) -> List<Z>:
    let n = max(hi - lo, 0)
    var out = List.filled(n, Z.zero)
    for i in 0..n: out[i] = p[lo + i]
    out

/// Multiplicative inverse `1 / p mod x^keep`; requires `p[0] != 0`.
pub fn inv_trunc(p: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    assert p.len > 0 and p[0] != 0
    var g = List.filled(keep, Z.zero)
    g[0] = p[0].inv
    var m = 1
    while m < keep:
        let size = min(m << 1, keep)
        let gm = sub_z(g, 0, m)
        let pg = convolve(sub_z(p, 0, min(p.len, size)), gm)
        let hi_end = min(pg.len, size)
        if hi_end > m:
            let rg = convolve(sub_z(pg, m, hi_end), gm)
            let lim = min(rg.len, size - m)
            for i in 0..lim: g[m + i] = -rg[i]
        m = size
    g

/// Formal logarithm `ln(p) mod x^keep`; requires `p[0] == 1`.
pub fn log_trunc(p: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    assert p.len > 0 and p[0].v == 1
    if keep == 1: return [Z.zero]
    pad_to(integral(head(convolve(derivative(head(p, keep)), inv_trunc(p, keep)), keep - 1)), keep)

/// Formal exponential `exp(p) mod x^keep`; requires `p[0] == 0`.
pub fn exp_trunc(p: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    assert p.len == 0 or p[0] == 0
    if keep <= 32:
        var g = List.filled(keep, Z.zero)
        g[0] = Z.one
        for k in 1..keep:
            var s = Z.zero
            for j in 1..min(k + 1, p.len): s += Z(j) * p[j] * g[k - j]
            g[k] = s / Z(k)
        return g
    var g = [Z.one]
    var m = 1
    while m < keep:
        let size = min(2 * m, keep)
        let lg = log_trunc(g, size)
        var diff = List.filled(size, Z.zero)
        for i in 0..size:
            let pi = if i < p.len: p[i] else: Z.zero
            diff[i] = pi - lg[i]
        diff[0] += Z.one
        g = head(convolve(g, diff), size)
        m *= 2
    g

/// Power `p^e mod x^keep`. Supports `e == 0`, positive `e` (including leading
/// zeros), and negative `e` (when `p[0] != 0`).
pub fn pow_trunc(p: List<Z>, e: Int, keep: Int) -> List<Z>:
    if keep == 0: return []
    if e == 0:
        var out = List.filled(keep, Z.zero)
        out[0] = Z.one
        return out
    var first = 0
    while first < p.len and p[first] == 0: first += 1
    if e < 0:
        assert p.len > 0 and first == 0
    elif first == p.len or (first > 0 and e > (keep - 1) / first):
        return List.filled(keep, Z.zero)
    let shift = first * e
    let rem = keep - shift
    let c = p[first]
    let ic = c.inv
    let n = min(p.len - first, rem)
    var q = List.filled(n, Z.zero)
    for i in 0..n: q[i] = p[first + i] * ic
    let lg = log_trunc(q, rem)
    let ez = Z(e)
    var scaled = List.filled(rem, Z.zero)
    for i in 0..rem: scaled[i] = lg[i] * ez
    let power = exp_trunc(scaled, rem)
    let scale = c.pow(e)
    var out = List.filled(keep, Z.zero)
    for i in 0..rem: out[shift + i] = power[i] * scale
    out

/// Arbitrary modular power `p(x)^alpha mod x^keep` for a monic series (`p[0] == 1`).
pub fn pow_mod_trunc(p: List<Z>, alpha: Z, keep: Int) -> List<Z>:
    if keep == 0: return []
    assert p.len > 0 and p[0].v == 1
    if alpha == 0:
        var out = List.filled(keep, Z.zero)
        out[0] = Z.one
        return out
    let lg = log_trunc(p, keep)
    var scaled = List.filled(keep, Z.zero)
    for i in 0..keep: scaled[i] = lg[i] * alpha
    exp_trunc(scaled, keep)

// Tonelli-Shanks square root mod P = 998244353 (returns min(r, P - r)).
fn sqrt_mod_z(a: Z) -> Z:
    if a == 0: return Z.zero
    let p = 998244353
    assert pow_mod_i(a.v, (p - 1) >> 1, p) == 1
    var q = p - 1
    var s = 0
    while (q & 1) == 0:
        q = q >> 1
        s += 1
    // 3 is a quadratic non-residue mod 998244353
    var c = pow_mod_i(3, q, p)
    var r = pow_mod_i(a.v, (q + 1) >> 1, p)
    var t = pow_mod_i(a.v, q, p)
    var m = s
    while t != 1:
        var i = 1
        var tt = (t * t) % p
        while tt != 1:
            tt = (tt * tt) % p
            i += 1
        let b = pow_mod_i(c, 1 << (m - i - 1), p)
        r = (r * b) % p
        c = (b * b) % p
        t = (t * c) % p
        m = i
    Z(min(r, p - r))

fn ext_gcd(a: Int, b: Int) -> (Int, Int, Int):
    if b == 0: return (a, 1, 0)
    let (g, x, y) = ext_gcd(b, a % b)
    (g, y, x - (a / b) * y)

/// Rational power `p(x)^(num / den) mod x^keep` (`den > 0`, `den % 998244353 != 0`).
/// Works for any `p[0] == 1`, any `den == 1`, any quadratic residue when `den == 2`,
/// and any `gcd(den, 998244352) == 1`; supports leading zeros when `num > 0` and `den | shift`.
pub fn pow_rat_trunc(p: List<Z>, num: Int, den: Int, keep: Int) -> List<Z>:
    assert den > 0 and den % 998244353 != 0
    if keep == 0: return []
    if num == 0:
        var out = List.filled(keep, Z.zero)
        out[0] = Z.one
        return out
    var first = 0
    while first < p.len and p[first] == 0: first += 1
    if first == p.len:
        assert num > 0
        return List.filled(keep, Z.zero)
    if first > 0:
        assert num > 0 and first % den == 0
        let step = first / den
        if step > (keep - 1) / num: return List.filled(keep, Z.zero)
        let shift = step * num
        let rem = keep - shift
        let sub = pow_rat_trunc([p[first + i] for i in 0..min(p.len - first, rem)], num, den, rem)
        var out = List.filled(keep, Z.zero)
        for i in 0..rem: out[shift + i] = sub[i]
        return out
    let c = p[0]
    let c_scale = if c.v == 1:
        Z.one
    elif den == 1:
        c.pow(num)
    elif den == 2:
        sqrt_mod_z(c).pow(num)
    else:
        let phi = 998244352
        let (g, inv_d, _) = ext_gcd(den % phi, phi)
        assert g == 1
        let e_root = ((inv_d % phi) + phi) % phi
        c.pow(e_root).pow(num)
    let ic = c.inv
    let n = min(p.len, keep)
    var q = List.filled(n, Z.zero)
    for i in 0..n: q[i] = p[i] * ic
    let alpha = Z(num) * Z(den).inv
    let base = pow_mod_trunc(q, alpha, keep)
    var out = List.filled(keep, Z.zero)
    for i in 0..keep: out[i] = base[i] * c_scale
    out

/// Formal square root `sqrt(p) mod x^keep`; chooses the smaller root of `p[0]` mod `998244353`.
pub fn sqrt_trunc(p: List<Z>, keep: Int) -> List<Z>:
    pow_rat_trunc(p, 1, 2, keep)

/// Euclidean polynomial division `(q, r)` with `a = q * b + r` and `r.len < b.len`.
pub fn div_mod(a: List<Z>, b: List<Z>) -> (List<Z>, List<Z>):
    let ta = trim_zeros(a)
    let tb = trim_zeros(b)
    assert tb.len > 0
    if ta.len < tb.len: return ([], ta)
    let q_len = ta.len - tb.len + 1
    let ra = rev_poly(ta)
    let rb = rev_poly(tb)
    let q_rev = head(convolve(head(ra, q_len), inv_trunc(rb, q_len)), q_len)
    let q = rev_poly(q_rev)
    let qb = convolve(q, tb)
    let r_len = tb.len - 1
    var r = List.filled(r_len, Z.zero)
    for i in 0..r_len:
        let ai = if i < ta.len: ta[i] else: Z.zero
        let qbi = if i < qb.len: qb[i] else: Z.zero
        r[i] = ai - qbi
    (trim_zeros(q), trim_zeros(r))

/// Product of all polynomials in `polys` in `O(S log^2 S)` where `S` is total degree.
pub fn prod_polys(polys: List<List<Z>>) -> List<Z>:
    if polys.len == 0: return [Z.one]
    if polys.len == 1: return polys[0]
    // Min-heap by polynomial length so unequal degrees balance optimally.
    var pool: List<List<Z>> = []
    var heap: List<Int> = []
    for p in polys:
        let id = pool.len
        pool.push(p)
        heap.push(id)
        var cur = heap.len - 1
        while cur > 0:
            let par = (cur - 1) >> 1
            if pool[heap[cur]].len < pool[heap[par]].len:
                heap.swap(cur, par)
                cur = par
            else:
                break
    while heap.len > 1:
        // pop smallest
        let a_id = heap[0]
        let last1 = heap.pop()
        if heap.len > 0:
            heap[0] = last1
            var cur = 0
            while true:
                let l = cur * 2 + 1
                let r = l + 1
                var best = cur
                if l < heap.len and pool[heap[l]].len < pool[heap[best]].len: best = l
                if r < heap.len and pool[heap[r]].len < pool[heap[best]].len: best = r
                if best == cur: break
                heap.swap(cur, best)
                cur = best
        // pop second smallest
        let b_id = heap[0]
        let last2 = heap.pop()
        if heap.len > 0:
            heap[0] = last2
            var cur = 0
            while true:
                let l = cur * 2 + 1
                let r = l + 1
                var best = cur
                if l < heap.len and pool[heap[l]].len < pool[heap[best]].len: best = l
                if r < heap.len and pool[heap[r]].len < pool[heap[best]].len: best = r
                if best == cur: break
                heap.swap(cur, best)
                cur = best
        let prod = convolve(pool[a_id], pool[b_id])
        let new_id = pool.len
        pool.push(prod)
        heap.push(new_id)
        var cur = heap.len - 1
        while cur > 0:
            let par = (cur - 1) >> 1
            if pool[heap[cur]].len < pool[heap[par]].len:
                heap.swap(cur, par)
                cur = par
            else:
                break
    pool[heap[0]]

/// Monic polynomial `prod_{c in pts} (x - c)` in `O(n log^2 n)`.
pub fn prod_linear(pts: List<Z>) -> List<Z>:
    if pts.len == 0: return [Z.one]
    SubproductTree.new(pts).product()

/// Subproduct tree over points `pts` supporting `O((n + m) log^2 m)` multipoint
/// evaluation (via transposed middle product) and `O(n log^2 n)` interpolation.
pub struct SubproductTree:
    n: Int
    pts: List<Z>
    nodes: List<List<Z>>

impl SubproductTree:
    /// Builds the product tree of `(x - pts[i])` in `O(n log^2 n)`.
    pub fn new(pts: List<Z>) -> SubproductTree:
        let n = pts.len
        if n == 0: return SubproductTree(n: 0, pts: [], nodes: [[Z.one]])
        var nodes: List<List<Z>> = List.filled(2 * n, [])
        for i in 0..n: nodes[n + i] = [-pts[i], Z.one]
        var i = n - 1
        while i >= 1:
            nodes[i] = convolve(nodes[2 * i], nodes[2 * i + 1])
            i -= 1
        SubproductTree(n: n, pts: pts, nodes: nodes)

    /// Root polynomial `prod_{i=0}^{n-1} (x - pts[i])`.
    pub fn product(self) -> List<Z>:
        if self.n == 0: return [Z.one]
        self.nodes[1]

    /// Evaluates `p` at every point in `self.pts` in `O((p.len + n) log^2 n)`.
    pub fn eval(self, p: List<Z>) -> List<Z>:
        let n = self.n
        if n == 0: return []
        if p.len == 0: return List.filled(n, Z.zero)
        let root = self.nodes[1]
        let reduced = if p.len > n: div_mod(p, root).1 else: p
        let r_pad = pad_to(reduced, n)
        let rev_r = rev_poly(r_pad)
        let inv_rev_m = inv_trunc(rev_poly(root), n)
        var down: List<List<Z>> = List.filled(2 * n, [])
        down[1] = head(convolve(rev_r, inv_rev_m), n)
        for i in 1..n:
            down[2 * i] = middle_product(down[i], self.nodes[2 * i + 1])
            down[2 * i + 1] = middle_product(down[i], self.nodes[2 * i])
        var out = List.filled(n, Z.zero)
        for i in 0..n: out[i] = down[n + i][0]
        out

    /// Returns the unique polynomial of degree `< n` with `p(pts[i]) == ys[i]`;
    /// requires all `pts` to be distinct and `ys.len == self.pts.len`.
    pub fn interpolate(self, ys: List<Z>) -> List<Z>:
        let n = self.n
        assert ys.len == n
        if n == 0: return []
        let derivs = self.eval(derivative(self.nodes[1]))
        // Batch-invert derivs[0..n] with Montgomery's trick
        var pref = List.filled(n + 1, Z.one)
        for i in 0..n:
            assert derivs[i] != 0
            pref[i + 1] = pref[i] * derivs[i]
        var inv_all = pref[n].inv
        var up: List<List<Z>> = List.filled(2 * n, [])
        var i = n - 1
        while i >= 0:
            let inv_d = inv_all * pref[i]
            inv_all *= derivs[i]
            up[n + i] = [ys[i] * inv_d]
            i -= 1
        var k = n - 1
        while k >= 1:
            let l = convolve(up[2 * k], self.nodes[2 * k + 1])
            let r = convolve(up[2 * k + 1], self.nodes[2 * k])
            up[k] = add_poly(l, r)
            k -= 1
        pad_to(up[1], n)

/// Evaluates `p` at all points in `xs` in `O((p.len + xs.len) log^2 xs.len)`.
pub fn multipoint_eval(p: List<Z>, xs: List<Z>) -> List<Z>:
    SubproductTree.new(xs).eval(p)

/// Fast `O(n log^2 n)` Lagrange interpolation over `Z`: returns degree `< n`
/// polynomial `p` with `p(xs[i]) == ys[i]`.
pub fn interpolate_fast(xs: List<Z>, ys: List<Z>) -> List<Z>:
    SubproductTree.new(xs).interpolate(ys)

/// Taylor shift `p(x + c)` in `O(n log n)`.
pub fn taylor_shift(p: List<Z>, c: Z) -> List<Z>:
    let n = p.len
    if n <= 1 or c == 0: return p
    let (fact, ifact) = build_facts(n)
    let mod_p = 998244353
    var a = List.filled(n, 0)
    for i in 0..n: a[n - 1 - i] = (p[i].v * fact[i]) % mod_p
    var b = List.filled(n, 0)
    var cp = 1
    for i in 0..n:
        b[i] = (cp * ifact[i]) % mod_p
        cp = (cp * c.v) % mod_p
    let prod = convolve_p1_raw(a, b)
    var out = List.filled(n, Z.zero)
    for k in 0..n:
        out[k] = Z((prod[n - 1 - k] * ifact[k]) % mod_p)
    out

// Non-overlapping helper for shift_eval: requires c + i not in {0, ..., n - 1} for all i in 0..m.
fn shift_eval_disjoint(ys: List<Z>, c: Z, m: Int) -> List<Z>:
    let n = ys.len
    if m == 0: return []
    if n == 0: return List.filled(m, Z.zero)
    let (_, ifact) = build_facts(n)
    let mod_p = 998244353
    var a = List.filled(n, 0)
    for i in 0..n:
        var v = (ys[i].v * ifact[i]) % mod_p
        v = (v * ifact[n - 1 - i]) % mod_p
        if ((n - 1 - i) & 1) == 1 and v != 0: v = mod_p - v
        a[i] = v
    let total = n + m - 1
    var offsets = List.filled(total, 0)
    for t in 0..total:
        offsets[t] = (c + Z(t - (n - 1))).v
    // Batch-invert offsets[0..total]
    var pref = List.filled(total + 1, 1)
    for t in 0..total:
        pref[t + 1] = (pref[t] * offsets[t]) % mod_p
    var inv_all = pow_mod_i(pref[total], mod_p - 2, mod_p)
    var inv_off = List.filled(total, 0)
    var t = total - 1
    while t >= 0:
        inv_off[t] = (inv_all * pref[t]) % mod_p
        inv_all = (inv_all * offsets[t]) % mod_p
        t -= 1
    let prod = convolve_p1_raw(a, inv_off)
    // Product factor for j: prod_{i=0}^{n-1} (c + j - i) = pref[j + n] * inv(pref[j])
    var inv_pref_m = List.filled(m, 0)
    if m > 0:
        var inv_p = pow_mod_i(pref[m - 1], mod_p - 2, mod_p)
        var j = m - 1
        while j >= 0:
            inv_pref_m[j] = inv_p
            if j > 0: inv_p = (inv_p * offsets[j - 1]) % mod_p
            j -= 1
    var out = List.filled(m, Z.zero)
    for j in 0..m:
        let factor = (pref[j + n] * inv_pref_m[j]) % mod_p
        out[j] = Z((factor * prod[j + n - 1]) % mod_p)
    out

/// Given `ys = [f(0), f(1), ..., f(n - 1)]` of a polynomial `f` of degree `< n`,
/// returns `[f(c), f(c + 1), ..., f(c + m - 1)]` in `O((n + m) log (n + m))`.
/// Handles any `c` including ranges overlapping `0..n`.
pub fn shift_eval(ys: List<Z>, c: Z, m: Int) -> List<Z>:
    let n = ys.len
    if m == 0: return []
    if n == 0: return List.filled(m, Z.zero)
    let mod_p = 998244353
    var out = List.filled(m, Z.zero)
    // Split 0..m into contiguous non-overlapping runs outside 0..n (mod P)
    var i = 0
    while i < m:
        let x = (c.v + i) % mod_p
        if x < n:
            out[i] = ys[x]
            i += 1
        else:
            let start = i
            // Run ends either at m or when (c.v + i) wraps around mod_p to 0
            let to_wrap = mod_p - x
            let run_len = min(m - start, to_wrap)
            let chunk = shift_eval_disjoint(ys, c + Z(start), run_len)
            for j in 0..run_len: out[start + j] = chunk[j]
            i += run_len
    out

/// Evaluates polynomial `p` at arithmetic progression `start + k * step` for `k = 0..count`.
pub fn eval_ap(p: List<Z>, start: Z, step: Z, count: Int) -> List<Z>:
    if count == 0: return []
    if p.len == 0: return List.filled(count, Z.zero)
    if step == 0:
        return List.filled(count, eval_poly(p, start))
    if count <= p.len:
        var xs = List.filled(count, Z.zero)
        var cur = start
        for k in 0..count:
            xs[k] = cur
            cur += step
        return multipoint_eval(p, xs)
    // Evaluate at the first p.len points, then extend in O(count log count) via shift_eval
    let d = p.len
    var xs = List.filled(d, Z.zero)
    var cur = start
    for k in 0..d:
        xs[k] = cur
        cur += step
    let init = multipoint_eval(p, xs)
    let rest = shift_eval(init, Z(d), count - d)
    var out = List.filled(count, Z.zero)
    for k in 0..d: out[k] = init[k]
    for k in 0..count - d: out[d + k] = rest[k]
    out

/// Chirp-Z transform: evaluates `p` at geometric progression `a * r^k` for `k = 0..count`
/// in `O((p.len + count) log (p.len + count))`.
pub fn eval_gp(p: List<Z>, a: Z, r: Z, count: Int) -> List<Z>:
    if count == 0: return []
    if p.len == 0: return List.filled(count, Z.zero)
    if r == 0:
        var out = List.filled(count, p[0])
        out[0] = eval_poly(p, a)
        return out
    let n = p.len
    let mod_p = 998244353
    // Scale p(a * x) first
    var q = List.filled(n, 0)
    var ap = 1
    for i in 0..n:
        q[i] = (p[i].v * ap) % mod_p
        ap = (ap * a.v) % mod_p
    // Bluestein: sum_{j=0}^{n-1} q[j] r^{k*j} = r^{C(k,2)} sum_j (q[j] r^{C(j,2)}) * r^{-C(k-j, 2)}?
    // Using r^{k*j} = r^{C(k+j,2) - C(k,2) - C(j,2)}:
    // ans[k] = r^{-C(k,2)} * sum_{j=0}^{n-1} (q[j] r^{-C(j,2)}) * r^{C(k+j, 2)}.
    let ir = r.inv.v
    let tot = n + count
    var r_tri = List.filled(tot, 1)
    var ir_tri = List.filled(tot, 1)
    var rp = 1
    var irp = 1
    for i in 1..tot:
        r_tri[i] = (r_tri[i - 1] * rp) % mod_p
        ir_tri[i] = (ir_tri[i - 1] * irp) % mod_p
        rp = (rp * r.v) % mod_p
        irp = (irp * ir) % mod_p
    // Let u[n - 1 - j] = q[j] * ir_tri[j], v[t] = r_tri[t] for 0 <= t < n + count - 1
    var u = List.filled(n, 0)
    for j in 0..n: u[n - 1 - j] = (q[j] * ir_tri[j]) % mod_p
    let v_len = n + count - 1
    var v = List.filled(v_len, 0)
    for t in 0..v_len: v[t] = r_tri[t]
    let prod = convolve_p1_raw(u, v)
    var out = List.filled(count, Z.zero)
    for k in 0..count:
        out[k] = Z((prod[n - 1 + k] * ir_tri[k]) % mod_p)
    out

// Advance Kinoshita-Li packed bivariate Q(x, y) by one doubling step.
// Returns tneg = DIF(Q(-x, y)) of length B = 4 << L.
fn biv_advance(mut c: List<Int>, l_big: Int, l_step: Int) -> List<Int>:
    let mod_p = 998244353
    let b_sz = 4 << l_big
    let half_b = 2 << l_big
    var tq = List.filled(b_sz, 0)
    for i in 0..b_sz: tq[i] = c[i]
    dif_p1(mut tq)
    // In DIF bit-reversed order, Q(-x, y) swaps adjacent pairs (2i, 2i + 1)
    var tn = List.filled(b_sz, 0)
    var prod = List.filled(b_sz, 0)
    var i = 0
    while i < b_sz:
        let u0 = tq[i]
        let u1 = tq[i + 1]
        tn[i] = u1
        tn[i + 1] = u0
        let r = (u0 * u1) % mod_p
        prod[i] = r
        prod[i + 1] = r
        i += 2
    dit_p1(mut prod)
    for j in 0..b_sz: c[j] = prod[j]
    for j in 1..half_b: c[j] = c[2 * j]
    let span = 2 << (l_big - l_step)
    let stride = 1 << (l_big - l_step)
    for j in 0..span:
        c[half_b + j] = c[j]
        c[j] = 0
    c[half_b] = if c[half_b] == 0: mod_p - 1 else: c[half_b] - 1
    c[0] = 1
    for j in half_b + stride..b_sz: c[j] = 0
    var off = 0
    while off < half_b:
        for j in 0..stride: c[off + stride + j] = 0
        off += span
    tn

/// Formal power series composition `f(g(x)) mod x^keep` in `O(keep log^2 keep)`
/// (Kinoshita-Li). Automatically handles `g[0] != 0` via `taylor_shift`.
pub fn compose_trunc(f: List<Z>, g: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    if f.len == 0: return List.filled(keep, Z.zero)
    if g.len > 0 and g[0] != 0:
        let f_shifted = taylor_shift(f, g[0])
        var g0 = pad_to(head(g, keep), min(g.len, keep))
        g0[0] = Z.zero
        return compose_trunc(f_shifted, g0, keep)
    if keep == 1: return [f[0]]
    let n = keep
    let m = min(f.len, n)
    let k = next_pow2(n)
    let l_big = ilog2_pow2(k)
    let b_sz = 4 * k
    let half_b = 2 * k
    let mod_p = 998244353
    var q_arr = List.filled(b_sz, 0)
    q_arr[0] = 1
    let g_lim = min(g.len, n)
    for i in 0..g_lim:
        let v = g[i].v
        q_arr[half_b + i] = if v == 0: 0 else: mod_p - v
    var tneg: List<List<Int>> = []
    for l_step in 1..=l_big:
        tneg.push(biv_advance(mut q_arr, l_big, l_step))
    // Base step: P = rev(f[0..m]) / Q_L(0, y) mod y^m
    var rev_f = List.filled(m, Z.zero)
    for i in 0..m: rev_f[i] = f[m - 1 - i]
    var ql = List.filled(m, Z.zero)
    let ql_lim = min(k + 1, m)
    for i in 0..ql_lim: ql[i] = Z(q_arr[2 * i])
    let p_base = head(convolve(rev_f, inv_trunc(ql, m)), m)
    var p_arr = List.filled(b_sz, 0)
    for i in 0..m: p_arr[k - m + i] = p_base[i].v
    var idx = k - 1
    while idx > 0:
        p_arr[2 * idx] = p_arr[idx]
        p_arr[idx] = 0
        idx -= 1
    var l_cur = l_big - 1
    while l_cur >= 0:
        let bit_mask = 1 << (l_big - l_cur)
        var j = half_b - 1
        while j > 0:
            let v = p_arr[j]
            p_arr[2 * j] = if ((2 * j) & bit_mask) != 0: 0 else: v
            p_arr[j] = 0
            j -= 1
        dif_p1(mut p_arr)
        let tn = tneg[l_cur]
        for j in 0..b_sz: p_arr[j] = (tn[j] * p_arr[j]) % mod_p
        dit_p1(mut p_arr)
        for j in 0..half_b:
            p_arr[j] = p_arr[half_b + j]
            p_arr[half_b + j] = 0
        l_cur -= 1
    var out = List.filled(n, Z.zero)
    for i in 0..n: out[i] = Z(p_arr[i])
    out

/// Power projection: returns `ans` of length `m` with
/// `ans[k] = sum_{i=0}^{w.len - 1} w[i] * [x^i] g(x)^k` for `0 <= k < m`,
/// in `O((w.len + m) log^2 (w.len + m))` (Kinoshita-Li). Handles any `g[0]`.
pub fn power_projection(g: List<Z>, w: List<Z>, m: Int) -> List<Z>:
    if m == 0: return []
    let n = w.len
    if n == 0: return List.filled(m, Z.zero)
    if g.len > 0 and g[0] != 0:
        let c = g[0].v
        var g0 = pad_to(head(g, n), min(g.len, n))
        g0[0] = Z.zero
        let base = power_projection(g0, w, m)
        let (fact, ifact) = build_facts(m)
        let mod_p = 998244353
        var a = List.filled(m, 0)
        var b = List.filled(m, 0)
        var cp = 1
        for k in 0..m:
            a[k] = (base[k].v * ifact[k]) % mod_p
            b[k] = (cp * ifact[k]) % mod_p
            cp = (cp * c) % mod_p
        let prod = convolve_p1_raw(a, b)
        var out = List.filled(m, Z.zero)
        for k in 0..m: out[k] = Z((prod[k] * fact[k]) % mod_p)
        return out
    if n == 1:
        var out = List.filled(m, Z.zero)
        out[0] = w[0]
        return out
    let k = next_pow2(n)
    let l_big = ilog2_pow2(k)
    let b_sz = 4 * k
    let half_b = 2 * k
    let mod_p = 998244353
    var p_arr = List.filled(b_sz, 0)
    for i in 0..n: p_arr[i] = w[n - 1 - i].v
    var q_arr = List.filled(b_sz, 0)
    q_arr[0] = 1
    let g_lim = min(g.len, n)
    for i in 0..g_lim:
        let v = g[i].v
        q_arr[half_b + i] = if v == 0: 0 else: mod_p - v
    var r_deg = n - 1
    for l_step in 1..=l_big:
        let tn = biv_advance(mut q_arr, l_big, l_step)
        dif_p1(mut p_arr)
        for j in 0..b_sz: p_arr[j] = (p_arr[j] * tn[j]) % mod_p
        dit_p1(mut p_arr)
        let parity = r_deg & 1
        r_deg = r_deg >> 1
        for j in 0..half_b: p_arr[j] = p_arr[2 * j + parity]
        for j in half_b..b_sz: p_arr[j] = 0
        let span = 2 << (l_big - l_step)
        let stride = 1 << (l_big - l_step)
        var off = 0
        while off < half_b:
            for j in 0..stride: p_arr[off + stride + j] = 0
            off += span
    var py = List.filled(k, Z.zero)
    for j in 0..k: py[j] = Z(p_arr[2 * j])
    var qy = List.filled(k + 1, Z.zero)
    for j in 0..=k: qy[j] = Z(q_arr[2 * j])
    let ans = head(convolve(py, inv_trunc(qy, m)), m)
    pad_to(ans, m)

/// Compositional inverse `h(x)` of `g(x)` modulo `x^keep` (`g(h(x)) = h(g(x)) = x mod x^keep`)
/// in `O(keep log^2 keep)` via Lagrange inversion; requires `g[0] == 0` and `g[1] != 0`.
pub fn compose_inv_trunc(g: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    if keep == 1: return [Z.zero]
    assert g.len >= 2 and g[0] == 0 and g[1] != 0
    let n = keep - 1
    let c = g[1]
    let ic = c.inv
    let g_lim = min(g.len, keep)
    var g_norm = List.filled(g_lim, Z.zero)
    for i in 0..g_lim: g_norm[i] = g[i] * ic
    var w = List.filled(keep, Z.zero)
    w[n] = Z.one
    let proj = power_projection(g_norm, w, keep)
    let (fact, ifact) = build_facts(n)
    let mod_p = 998244353
    var f_ser = List.filled(n, Z.zero)
    f_ser[0] = Z.one
    for k in 1..n:
        let inv_k = (ifact[k] * fact[k - 1]) % mod_p
        f_ser[n - k] = proj[k] * Z((n * inv_k) % mod_p)
    let alpha = -Z(n).inv
    let h_div_x = pow_mod_trunc(f_ser, alpha, n)
    var out = List.filled(keep, Z.zero)
    var ic_pow = ic
    for k in 0..n:
        out[k + 1] = h_div_x[k] * ic_pow
        ic_pow *= ic
    out

/// Bostan-Mori: returns `[x^k] (p(x) / q(x))` in `O(d log d log k)` where
/// `d = max(p.len, q.len)`; requires `q[0] != 0` and `k >= 0`.
pub fn kth_term(p: List<Z>, q: List<Z>, k: Int) -> Z:
    assert q.len > 0 and q[0] != 0 and k >= 0
    if p.len == 0: return 0
    let mod_p = Z.MOD
    var cur_p = [x.v for x in p]
    var cur_q = [x.v for x in q]
    var kk = k
    while kk > 0:
        let nq = cur_q.len
        var q_neg = List.filled(nq, 0)
        for i in 0..nq:
            let v = cur_q[i]
            q_neg[i] = if (i & 1) == 1 and v != 0: mod_p - v else: v
        let num = convolve_p1_raw(cur_p, q_neg)
        let den = convolve_p1_raw(cur_q, q_neg)
        let parity = kk & 1
        let np = (num.len - parity + 1) / 2
        let nd = (den.len + 1) / 2
        cur_p = [num[parity + 2 * i] for i in 0..np]
        cur_q = [den[2 * j] for j in 0..nd]
        kk = kk >> 1
        if cur_p.len == 0: return 0
    Z(cur_p[0]) * Z(cur_q[0]).inv

/// Convolution of arbitrary-modulus integer sequences modulo `m` via 3-prime NTT + Garner CRT.
/// Works for any `m >= 1` up to `10^9 + 7` (and beyond, as long as `m * m < 2^62`).
pub fn convolve_mod(a: List<Int>, b: List<Int>, m: Int) -> List<Int>:
    assert m > 0
    if a.len == 0 or b.len == 0: return []
    let na0 = a.len
    let nb0 = b.len
    let n_out = na0 + nb0 - 1
    if m == 1: return List.filled(n_out, 0)
    // Trim leading and trailing zeros (modulo m) so shifted/sparse inputs avoid wasted NTT work.
    var a_lo = 0
    while a_lo < na0 and (a[a_lo] == 0 or (a[a_lo] % m) == 0): a_lo += 1
    if a_lo == na0: return List.filled(n_out, 0)
    var a_hi = na0
    while a_hi > a_lo and (a[a_hi - 1] == 0 or (a[a_hi - 1] % m) == 0): a_hi -= 1
    var b_lo = 0
    while b_lo < nb0 and (b[b_lo] == 0 or (b[b_lo] % m) == 0): b_lo += 1
    if b_lo == nb0: return List.filled(n_out, 0)
    var b_hi = nb0
    while b_hi > b_lo and (b[b_hi - 1] == 0 or (b[b_hi - 1] % m) == 0): b_hi -= 1
    let na = a_hi - a_lo
    let nb = b_hi - b_lo
    let n = na + nb - 1
    let shift = a_lo + b_lo
    var ra = List.filled(na, 0)
    for i in 0..na:
        let x = a[a_lo + i]
        if x >= 0 and x < m:
            ra[i] = x
        else:
            var v = x % m
            if v < 0: v += m
            ra[i] = v
    var rb = List.filled(nb, 0)
    for i in 0..nb:
        let x = b[b_lo + i]
        if x >= 0 and x < m:
            rb[i] = x
        else:
            var v = x % m
            if v < 0: v += m
            rb[i] = v
    if min(na, nb) <= 80:
        var c = [0..n] _ => 0
        if m <= 1073741824:
            let big = 7 * m * m
            for i in 0..na:
                let ai = ra[i]
                if ai == 0: continue
                for j in 0..nb:
                    let v = c[i + j] + ai * rb[j]
                    c[i + j] = if v >= big: v - big else: v
        else:
            for i in 0..na:
                let ai = ra[i]
                if ai == 0: continue
                for j in 0..nb:
                    c[i + j] = (c[i + j] + ai * rb[j]) % m
        var out = List.filled(n_out, 0)
        for i in 0..n: out[shift + i] = c[i] % m
        return out
    let mode = detect_sym_mode(ra, rb)
    let c1 = conv_core(ra, rb, n, mode, 998244353)
    let c2 = conv_core(ra, rb, n, mode, 469762049)
    let c3 = conv_core(ra, rb, n, mode, 167772161)
    // Garner CRT with Montgomery reductions for P2 and P3 (one `% m` per term).
    let p1 = 998244353
    let p2 = 469762049
    let p3 = 167772161
    let p1_m = p1 % m
    let p12_m = (p1_m * (p2 % m)) % m
    var out = List.filled(n_out, 0)
    for i in 0..n:
        let v1 = c1[i]
        let v1_p2 = if v1 >= 939524098: v1 - 939524098 elif v1 >= p2: v1 - p2 else: v1
        var d2 = c2[i] - v1_p2
        if d2 < 0: d2 += p2
        let x2 = d2 * 305718167
        var t2 = (x2 + (((x2 & 4294967295) * 469762047) & 4294967295) * p2) >> 32
        if t2 >= p2: t2 -= p2
        let xv1 = v1 * 100663271
        let sv1 = (xv1 + (((xv1 & 4294967295) * 167772159) & 4294967295) * p3) >> 32
        let xt2 = t2 * 80530764
        let st2 = (xt2 + (((xt2 & 4294967295) * 167772159) & 4294967295) * p3) >> 32
        let d3 = c3[i] + 671088644 - sv1 - st2
        let x3 = d3 * 115614034
        var t3 = (x3 + (((x3 & 4294967295) * 167772159) & 4294967295) * p3) >> 32
        if t3 >= p3: t3 -= p3
        out[shift + i] = (v1 + t2 * p1_m + t3 * p12_m) % m
    out

/// Horner evaluation of `p` at `x` over `Mod<P>`.
pub fn eval_poly<P>(p: List<Mod<P>>, x: Mod<P>) -> Mod<P>:
    var acc = Mod<P>.zero
    var i = p.len - 1
    while i >= 0:
        acc = acc * x + p[i]
        i -= 1
    acc

/// The unique polynomial of degree `< pts.len` passing through `pts`
/// in `O(n^2)` over any prime modulus `P`; requires distinct `x`-coordinates.
pub fn interpolate<P>(pts: List<(Mod<P>, Mod<P>)>) -> List<Mod<P>>:
    let n = pts.len
    var ans = List.filled(n, Mod<P>.zero)
    // prod(x) = product_{k} (x - x_k)
    var prod = [Mod<P>.one]
    for (xi, _) in pts:
        var next = List.filled(prod.len + 1, Mod<P>.zero)
        for k in 0..prod.len:
            next[k] -= prod[k] * xi
            next[k + 1] += prod[k]
        prod = next
    for i in 0..n:
        let (xi, yi) = pts[i]
        var denom = Mod<P>.one
        for j in 0..n:
            if i != j: denom *= xi - pts[j].0
        let scale = yi / denom
        // synthetic division of prod by (x - xi), highest degree first
        var rem = Mod<P>.zero
        var k = n - 1
        while k >= 0:
            rem = prod[k + 1] + rem * xi
            ans[k] += rem * scale
            k -= 1
    ans

// ---- tests ----

fn coef(p: List<Z>, i: Int) -> Z:
    if i < p.len: p[i] else: Z.zero

fn slow_mul(a: List<Z>, b: List<Z>, keep: Int) -> List<Z>:
    var out = List.filled(keep, Z.zero)
    for i in 0..min(a.len, keep):
        for j in 0..min(b.len, keep - i):
            out[i + j] += a[i] * b[j]
    out

fn slow_inv(p: List<Z>, keep: Int) -> List<Z>:
    var g = List.filled(keep, Z.zero)
    if keep == 0: return g
    let inv0 = p[0].inv
    g[0] = inv0
    for k in 1..keep:
        var s = Z.zero
        for j in 1..=k: s += coef(p, j) * g[k - j]
        g[k] = -s * inv0
    g

fn slow_log(p: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    let inv = slow_inv(p, keep)
    let d = [coef(p, k + 1) * Z(k + 1) for k in 0..keep - 1]
    let q = slow_mul(d, inv, keep - 1)
    [if k == 0: Z.zero else: q[k - 1] / Z(k) for k in 0..keep]

fn slow_exp(p: List<Z>, keep: Int) -> List<Z>:
    var g = List.filled(keep, Z.zero)
    if keep == 0: return g
    g[0] = Z.one
    for k in 1..keep:
        var s = Z.zero
        for j in 1..=k: s += Z(j) * coef(p, j) * g[k - j]
        g[k] = s / Z(k)
    g

fn slow_pow(p: List<Z>, e: Int, keep: Int) -> List<Z>:
    if keep == 0: return []
    if e < 0: return slow_pow(slow_inv(p, keep), -e, keep)
    var res = List.filled(keep, Z.zero)
    res[0] = Z.one
    var base = head(p, keep)
    var exp = e
    while exp > 0:
        if exp & 1 == 1: res = slow_mul(res, base, keep)
        base = slow_mul(base, base, keep)
        exp /= 2
    res

fn slow_compose(f: List<Z>, g: List<Z>, keep: Int) -> List<Z>:
    if keep == 0: return []
    var res = List.filled(keep, Z.zero)
    var g_pow = List.filled(keep, Z.zero)
    g_pow[0] = Z.one
    let gt = head(g, keep)
    for k in 0..f.len:
        for i in 0..keep: res[i] += f[k] * g_pow[i]
        g_pow = slow_mul(g_pow, gt, keep)
    res

fn slow_power_projection(g: List<Z>, w: List<Z>, m: Int) -> List<Z>:
    if m == 0: return []
    let n = w.len
    var out = List.filled(m, Z.zero)
    if n == 0: return out
    var g_pow = List.filled(n, Z.zero)
    g_pow[0] = Z.one
    let gt = head(g, n)
    for k in 0..m:
        var acc = Z.zero
        for i in 0..n: acc += w[i] * g_pow[i]
        out[k] = acc
        g_pow = slow_mul(g_pow, gt, n)
    out

fn slow_taylor_shift(p: List<Z>, c: Z) -> List<Z>:
    let n = p.len
    if n == 0: return []
    var res = List.filled(n, Z.zero)
    let xc = [c, Z.one]
    var pow = [Z.one]
    for k in 0..n:
        for i in 0..pow.len: res[i] += p[k] * pow[i]
        pow = slow_mul(pow, xc, n)
    res

fn slow_middle_product(a: List<Z>, b: List<Z>) -> List<Z>:
    if b.len == 0 or a.len < b.len: return []
    let out_len = a.len - b.len + 1
    var out = List.filled(out_len, Z.zero)
    for i in 0..out_len:
        var s = Z.zero
        for j in 0..b.len: s += a[i + j] * b[j]
        out[i] = s
    out

fn lcg_next(seed: Int) -> Int:
    (seed * 1103515245 + 12345) & 2147483647

fn table_matches<P>(rank2: Int) -> Bool:
    let r_mod = 4294967296 % P
    var rt = List.filled(rank2 + 1, 0)
    var irt = List.filled(rank2 + 1, 0)
    rt[rank2] = pow_mod_i(3, (P - 1) >> rank2, P)
    irt[rank2] = pow_mod_i(rt[rank2], P - 2, P)
    for k in (0..rank2).rev:
        rt[k] = (rt[k + 1] * rt[k + 1]) % P
        irt[k] = (irt[k + 1] * irt[k + 1]) % P
    if rt[0] != 1 or rt[1] != P - 1: return false
    var (prod, iprod) = (1, 1)
    var want: List<Int> = []
    var iwant: List<Int> = []
    for j in 0..rank2 - 2:
        want.push((((rt[j + 3] * prod) % P) * r_mod) % P)
        iwant.push((((irt[j + 3] * iprod) % P) * r_mod) % P)
        prod = (prod * irt[j + 3]) % P
        iprod = (iprod * rt[j + 3]) % P
    let roots = build_roots(P)
    roots.r3 == want and roots.ir3 == iwant

@test fn step_root_tables_match_direct_computation():
    assert table_matches::<998244353>(23)
    assert table_matches::<469762049>(26)
    assert table_matches::<167772161>(25)

@test fn the_transform_round_trips():
    var s = 42
    for n in [1, 2, 4, 8, 16, 32, 64, 128]:
        var a: List<Z> = []
        for _ in 0..n:
            s = lcg_next(s)
            a.push(Z(s))
        var b = a.copy()
        ntt(mut b, false)
        ntt(mut b, true)
        assert a == b

@test fn convolution_matches_the_schoolbook_product():
    var s = 7
    for (n, m) in [(1, 1), (3, 4), (17, 23), (35, 40), (64, 64), (70, 55)]:
        var a: List<Z> = []
        var b: List<Z> = []
        for _ in 0..n:
            s = lcg_next(s)
            a.push(Z(s))
        for _ in 0..m:
            s = lcg_next(s)
            b.push(Z(s))
        let got = convolve(a, b)
        assert got == slow_mul(a, b, n + m - 1)
        // Squaring fast path
        assert convolve(a, a) == slow_mul(a, a, 2 * n - 1)
        // Autocorrelation fast path (b == rev(a))
        let ra = rev_poly(a)
        assert convolve(a, ra) == slow_mul(a, ra, 2 * n - 1)
    assert convolve([], [Z.one]) == []

@test fn counting_dice_sums():
    let die: List<Z> = [0, 1, 1, 1, 1, 1, 1]
    let ways = pow_trunc(die, 20, 121)
    assert ways[19] == 0 and ways[20] == 1 and ways[120] == 1
    let six: Z = 6
    assert eval_poly(ways, 1) == six.pow(20)

@test fn mtt_arbitrary_modulus_and_interpolation():
    var s = 99
    for m in [1, 7, 1000000007, 1000000000]:
        for (na, nb) in [(12, 15), (40, 40), (45, 38)]:
            var a: List<Int> = []
            var b: List<Int> = []
            for _ in 0..na:
                s = lcg_next(s)
                a.push(s % m - m / 2)
            for _ in 0..nb:
                s = lcg_next(s)
                b.push(s % m - m / 2)
            let got = convolve_mod(a, b, m)
            var want = List.filled(na + nb - 1, 0)
            for i in 0..na:
                for j in 0..nb:
                    want[i + j] = (want[i + j] + a[i] * b[j]) % m
            for k in 0..want.len:
                if want[k] < 0: want[k] += m
            assert got == want
            // Squaring & autocorrelation fast paths in convolve_mod
            var ra = List.filled(na, 0)
            for i in 0..na: ra[i] = a[na - 1 - i]
            let got_sq = convolve_mod(a, a, m)
            let got_ac = convolve_mod(a, ra, m)
            var want_sq = List.filled(2 * na - 1, 0)
            var want_ac = List.filled(2 * na - 1, 0)
            for i in 0..na:
                for j in 0..na:
                    want_sq[i + j] = (want_sq[i + j] + a[i] * a[j]) % m
                    want_ac[i + j] = (want_ac[i + j] + a[i] * ra[j]) % m
            for k in 0..2 * na - 1:
                if want_sq[k] < 0: want_sq[k] += m
                if want_ac[k] < 0: want_ac[k] += m
            assert got_sq == want_sq and got_ac == want_ac
    let hidden: List<Mod<1000000007>> = [3, -2, 5, 1]
    let pts = [(Mod<1000000007>(x), eval_poly(hidden, Mod<1000000007>(x))) for x in [0, 2, 5, 11]]
    assert interpolate(pts) == hidden

@test fn derivative_and_integral_undo_each_other():
    let p: List<Z> = [7, 3, -5, 11, 2]
    assert derivative(integral(p)) == p
    let want_id: List<Z> = [0, 3, -5, 11, 2]
    assert integral(derivative(p)) == want_id
    let p9: List<Z> = [9]
    assert derivative([]) == [] and derivative(p9) == [] and integral([]) == []

@test fn inverse_matches_the_recurrence():
    let one_minus_x: List<Z> = [1, -1]
    assert inv_trunc(one_minus_x, 40) == List.filled(40, Z.one)
    let two: Z = 2
    assert inv_trunc([two], 3) == [two.inv, 0, 0] and inv_trunc([two], 0) == []
    var s = 123
    for (len, keep) in [(1, 8), (5, 20), (16, 16), (35, 70), (70, 45)]:
        var p: List<Z> = []
        for i in 0..len:
            s = lcg_next(s)
            p.push(Z(if i == 0: (s % 1000) + 1 else: s))
        let got = inv_trunc(p, keep)
        assert got == slow_inv(p, keep)
        let prod = slow_mul(p, got, keep)
        assert prod == [if i == 0: Z.one else: Z.zero for i in 0..keep]
        assert inv_trunc([coef(p, i) for i in 0..len + 5], keep) == got

@test fn log_and_exp_match_their_recurrences():
    let harmonic = log_trunc(List.filled(40, Z.one), 40)
    let e_x = exp_trunc([Z.zero, Z.one], 40)
    var fact = Z.one
    for k in 0..40:
        if k > 0:
            fact *= Z(k)
            assert harmonic[k] * Z(k) == Z.one
        assert e_x[k] * fact == Z.one
    assert exp_trunc([], 5) == [Z.one, Z.zero, Z.zero, Z.zero, Z.zero]
    assert exp_trunc([Z.zero, Z.zero], 3) == [Z.one, Z.zero, Z.zero]
    assert log_trunc([Z.one], 4) == [Z.zero, Z.zero, Z.zero, Z.zero]
    let p15: List<Z> = [1, 5]
    let p05: List<Z> = [0, 5]
    assert log_trunc(p15, 0) == [] and exp_trunc(p05, 0) == []
    var s = 321
    for keep in [1, 2, 7, 17, 35, 50]:
        var p = [Z.one]
        var q = [Z.zero]
        for _ in 1..keep:
            s = lcg_next(s)
            p.push(Z(s))
            s = lcg_next(s)
            q.push(Z(s))
        let lg = log_trunc(p, keep)
        assert lg == slow_log(p, keep)
        assert exp_trunc(lg, keep) == head(p, keep)
        let ex = exp_trunc(q, keep)
        assert ex == slow_exp(q, keep)
        assert log_trunc(ex, keep) == head(q, keep)

@test fn power_matches_repeated_multiplication_including_negative():
    var s = 555
    let cases: List<(List<Z>, Int)> = [([2, 3, -1, 5], 40), ([1, 4, 7], 35), ([3], 10)]
    for (p, keep) in cases:
        for e in [-10, -3, -1, 0, 1, 2, 5, 11]:
            assert pow_trunc(p, e, keep) == slow_pow(p, e, keep)
    // Negative exponent inverse check: p^e * p^{-e} == 1
    var p: List<Z> = []
    for i in 0..45:
        s = lcg_next(s)
        p.push(Z(if i == 0: 7 else: s))
    assert slow_mul(pow_trunc(p, -13, 45), pow_trunc(p, 13, 45), 45) == [if i == 0: Z.one else: Z.zero for i in 0..45]
    // Large negative exponent: (1 + x)^(-N) mod x^keep
    let big_neg = -1000000007
    let got_neg = pow_trunc([Z.one, Z.one], big_neg, 20)
    var binom = Z.one
    for k in 0..20:
        assert got_neg[k] == binom
        binom *= Z(big_neg - k) / Z(k + 1)
    // Leading zeros with non-negative exponents
    let p_lz: List<Z> = [0, 0, 2, 1]
    let want_lz: List<Z> = [0, 0, 0, 0, 0, 0, 8, 12, 6]
    assert pow_trunc(p_lz, 3, 9) == want_lz
    assert pow_trunc([], 0, 3) == [Z.one, Z.zero, Z.zero] and pow_trunc([], 4, 3) == [Z.zero, Z.zero, Z.zero]
    assert pow_trunc([Z.zero, Z.zero], 0, 2) == [Z.one, Z.zero] and pow_trunc([Z.zero, Z.zero], 1, 2) == [Z.zero, Z.zero]
    let e = 1000000000000
    let got = pow_trunc([Z.one, Z.one], e, 20)
    var b2 = Z.one
    for k in 0..20:
        assert got[k] == b2
        b2 *= Z(e - k) / Z(k + 1)
    let three: Z = 3
    assert pow_trunc([0, three], e, 20) == List.filled(20, Z.zero)
    assert pow_trunc([three], e, 2) == [three.pow(e), 0]

@test fn modular_and_rational_powers_and_sqrt():
    var s = 777
    // Check (p^(num/den))^den == p^num for positive & negative num, den in {1, 2, 3, 5}
    for (num, den) in [(1, 2), (-1, 2), (3, 2), (1, 3), (-2, 3), (2, 5), (-3, 5)]:
        var p = [Z.one]
        for _ in 1..40:
            s = lcg_next(s)
            p.push(Z(s))
        let r = pow_rat_trunc(p, num, den, 40)
        assert slow_pow(r, den, 40) == slow_pow(p, num, 40)
        // Matches pow_mod_trunc
        assert r == pow_mod_trunc(p, Z(num) / Z(den), 40)
    // Non-monic sqrt_trunc (p[0] = 9 -> root 3, p[0] = 49 -> root 7, and leading zeros!)
    let p9: List<Z> = [9, 6, 7, -2, 11, 4]
    let sq9 = sqrt_trunc(p9, 36)
    assert sq9[0] == 3 and slow_mul(sq9, sq9, 36) == pad_to(p9, 36)
    let p_shift: List<Z> = [0, 0, 4, 12, 9, 5]
    let sq_shift = sqrt_trunc(p_shift, 20)
    assert sq_shift[0] == 0 and sq_shift[1] == 2
    assert slow_mul(sq_shift, sq_shift, 20) == pad_to(p_shift, 20)
    // Non-monic cube root (gcd(3, 998244352) == 1)
    let p8: List<Z> = [8, 12, 6, 1, 5]
    let cb = pow_rat_trunc(p8, 1, 3, 25)
    assert slow_pow(cb, 3, 25) == pad_to(p8, 25)

@test fn middle_product_and_div_mod_match_naive():
    var s = 888
    for (na, nb) in [(1, 1), (5, 3), (16, 16), (40, 20), (75, 40), (90, 35)]:
        var a: List<Z> = []
        var b: List<Z> = []
        for _ in 0..na:
            s = lcg_next(s)
            a.push(Z(s))
        for _ in 0..nb:
            s = lcg_next(s)
            b.push(Z(s))
        assert middle_product(a, b) == slow_middle_product(a, b)
        let (q, r) = div_mod(a, b)
        assert r.len < b.len
        assert trim_zeros(add_poly(convolve(q, b), r)) == trim_zeros(a)
    let mp_a: List<Z> = [1, 2]
    let mp_b: List<Z> = [1, 2, 3]
    assert middle_product(mp_a, mp_b) == []
    assert middle_product([Z.one], []) == []

@test fn subproduct_tree_eval_interpolate_and_prod_polys():
    var s = 1001
    for (deg_p, n_pts) in [(0, 5), (5, 1), (10, 10), (35, 20), (20, 40), (50, 50)]:
        var p: List<Z> = []
        for _ in 0..deg_p:
            s = lcg_next(s)
            p.push(Z(s))
        var xs: List<Z> = []
        for i in 0..n_pts:
            xs.push(Z(i * 17 + 3))
        let tree = SubproductTree.new(xs)
        // Check root polynomial matches naive product of (x - xs[i])
        var want_root = [Z.one]
        for x in xs: want_root = slow_mul(want_root, [-x, Z.one], want_root.len + 1)
        assert tree.product() == want_root
        assert prod_linear(xs) == want_root
        // Check multipoint evaluation matches Horner
        let got_ys = tree.eval(p)
        let want_ys = [eval_poly(p, x) for x in xs]
        assert got_ys == want_ys
        assert multipoint_eval(p, xs) == want_ys
        // Check fast interpolation recovers exact polynomial when deg_p <= n_pts
        if deg_p <= n_pts:
            let rec = tree.interpolate(want_ys)
            assert rec == pad_to(p, n_pts)
            assert interpolate_fast(xs, want_ys) == pad_to(p, n_pts)
    // prod_polys with unbalanced degrees
    var polys: List<List<Z>> = []
    var want_prod = [Z.one]
    for sz in [1, 2, 15, 3, 25, 4, 8, 2]:
        var q: List<Z> = []
        for _ in 0..sz:
            s = lcg_next(s)
            q.push(Z(s))
        polys.push(q)
        want_prod = slow_mul(want_prod, q, want_prod.len + sz - 1)
    assert prod_polys(polys) == want_prod
    assert prod_polys([]) == [Z.one]

@test fn taylor_shift_matches_naive():
    var s = 2002
    for n in [0, 1, 2, 5, 16, 40]:
        var p: List<Z> = []
        for _ in 0..n:
            s = lcg_next(s)
            p.push(Z(s))
        let shifts: List<Z> = [0, 1, -5, 123456789]
        for c in shifts:
            let shifted = taylor_shift(p, c)
            assert shifted == slow_taylor_shift(p, c)
            // Inverse shift returns original polynomial
            assert taylor_shift(shifted, -c) == p

@test fn shift_eval_ap_and_gp_match_horner():
    var s = 3003
    for n in [1, 2, 7, 20, 40]:
        var p: List<Z> = []
        for _ in 0..n:
            s = lcg_next(s)
            p.push(Z(s))
        let ys = [eval_poly(p, Z(i)) for i in 0..n]
        // Non-overlapping, overlapping, wrapping, and negative c!
        let sh_cases: List<(Z, Int)> = [(Z(n + 5), 45), (0, n + 15), (-5, 25), (Z(n / 2), n), (123456, 35)]
        for (c, m) in sh_cases:
            let got = shift_eval(ys, c, m)
            let want = [eval_poly(p, c + Z(j)) for j in 0..m]
            assert got == want
        // eval_ap with arbitrary start, step (including 0), count < n and count > n
        let ap_cases: List<(Z, Z, Int)> = [(7, 13, 10), (-3, 5, n + 20), (42, 0, 6)]
        for (st, step, cnt) in ap_cases:
            let got_ap = eval_ap(p, st, step, cnt)
            let want_ap = [eval_poly(p, st + Z(k) * step) for k in 0..cnt]
            assert got_ap == want_ap
        // eval_gp (Chirp-Z) including r == 0, r == 1, arbitrary r
        let gp_cases: List<(Z, Z, Int)> = [(3, 5, 35), (1, 1, 10), (7, 0, 5), (11, -9, n + 15)]
        for (a, r, cnt) in gp_cases:
            let got_gp = eval_gp(p, a, r, cnt)
            var want_gp: List<Z> = []
            var rp = Z.one
            for k in 0..cnt:
                want_gp.push(eval_poly(p, a * rp))
                rp *= r
            assert got_gp == want_gp

@test fn composition_power_projection_and_compositional_inverse():
    var s = 4004
    for keep in [0, 1, 2, 3, 4, 5, 8, 9, 16, 17, 35]:
        var f: List<Z> = []
        var g0: List<Z> = []
        var g_nz: List<Z> = []
        var w: List<Z> = []
        for i in 0..keep:
            s = lcg_next(s)
            f.push(Z(s))
            s = lcg_next(s)
            g0.push(if i == 0: Z.zero else: Z(s))
            g_nz.push(if i == 0: 7 else: Z(s))
            s = lcg_next(s)
            w.push(Z(s))
        // Composition with g[0] == 0 and g[0] != 0
        assert compose_trunc(f, g0, keep) == slow_compose(f, g0, keep)
        assert compose_trunc(f, g_nz, keep) == slow_compose(f, g_nz, keep)
        // Shorter f than keep
        if keep >= 4:
            let f_short = head(f, keep / 2)
            assert compose_trunc(f_short, g0, keep) == slow_compose(f_short, g0, keep)
        // Power projection with m < keep, m == keep, m > keep, and g[0] != 0
        for m in [0, 1, keep, keep + 5]:
            assert power_projection(g0, w, m) == slow_power_projection(g0, w, m)
            assert power_projection(g_nz, w, m) == slow_power_projection(g_nz, w, m)
        // Compositional inverse when keep >= 2 and g[1] != 0
        if keep >= 2:
            var gi = g0.copy()
            if gi[1] == 0: gi[1] = 3
            let h = compose_inv_trunc(gi, keep)
            var want_id = List.filled(keep, Z.zero)
            want_id[1] = Z.one
            assert compose_trunc(gi, h, keep) == want_id
            assert compose_trunc(h, gi, keep) == want_id
    // Classic Catalan check (CF 2255 F pattern): C(x) = (1 - sqrt(1 - 4x)) / (2x), C(x) = 1 + x C(x)^2
    let keep_c = 25
    let one_m4x: List<Z> = [1, -4]
    let sq = sqrt_trunc(one_m4x, keep_c + 1)
    let two_c: Z = 2
    let inv2 = two_c.inv
    let cat = [(Z.zero - sq[i + 1]) * inv2 for i in 0..keep_c]
    let x_cat = [if i == 0: Z.zero else: cat[i - 1] for i in 0..keep_c]
    // Compositional inverse of x * C(x) is x * (1 - x) = [0, 1, -1]!
    assert compose_inv_trunc(x_cat, keep_c) == [if i == 1: Z.one elif i == 2: -Z.one else: Z.zero for i in 0..keep_c]

@test fn bostan_mori_kth_term_matches_inv_trunc():
    // Fibonacci: F(x) = x / (1 - x - x^2)
    let p_fib: List<Z> = [0, 1]
    let q_fib: List<Z> = [1, -1, -1]
    var a = Z.zero
    var b = Z.one
    for k in 0..25:
        assert kth_term(p_fib, q_fib, k) == a
        let c = a + b
        a = b
        b = c
    // General rational function vs inv_trunc up to k = 50
    let p: List<Z> = [3, -2, 5, 1]
    let q: List<Z> = [2, 1, -3, 4, -1]
    let series = head(convolve(p, inv_trunc(q, 55)), 55)
    for k in [0, 1, 2, 7, 15, 31, 50]:
        assert kth_term(p, q, k) == series[k]

@test fn radix7_and_symmetry_convolutions_match_schoolbook():
    var s = 9001
    // Exercise even-height, odd-height power-of-2, and radix-7 (h=9: h2=6, h=11: h2=8)
    // paths for autocorrelation, squaring, and general convolution.
    for na in [90, 130, 192, 224, 225]:
        var a: List<Z> = []
        var b: List<Z> = []
        for _ in 0..na:
            s = lcg_next(s)
            a.push(Z(s))
            s = lcg_next(s)
            b.push(Z(s))
        let a_rev = [a[na - 1 - i] for i in 0..na]
        let out_n = 2 * na - 1
        assert convolve(a, a_rev) == slow_mul(a, a_rev, out_n)
        assert convolve(a, a) == slow_mul(a, a, out_n)
        assert convolve(a, b) == slow_mul(a, b, out_n)
    // Exercise h=11 (M=256, h2=8 where h2 % 3 == 2) radix-7 autocorrelation
    for na in [600, 800]:
        var a: List<Z> = []
        for _ in 0..na:
            s = lcg_next(s)
            a.push(Z(s))
        let a_rev = [a[na - 1 - i] for i in 0..na]
        let got = convolve(a, a_rev)
        for idx in [0, 1, 17, 255, 256, 511, 512, na - 1, na, 2 * na - 2]:
            var want = Z.zero
            let lo = if idx - (na - 1) > 0: idx - (na - 1) else: 0
            let hi = if idx < na - 1: idx else: na - 1
            for i in lo..=hi:
                want += a[i] * a_rev[idx - i]
            assert got[idx] == want
