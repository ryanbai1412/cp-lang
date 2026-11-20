// COI 2007 #1, in the forms proto/cp.py lib-run takes today (proto/cpc has
// no floating point, so this one runs on the interpreter). See ../coi07p1.cp.
// original: corpus/submissions/5323995.cpp
let k: F64 = read()
let n: Int = read()
var t = 0.0
var p = 0.0
for i in 0..n:
    let x: F64 = read()
    if i == 0: p = x - k
    if abs(p + k - x) <= t: p += k
    elif x <= p + k: p = x + t
    else:
        let more = (x - t - p - k) / 2.0
        p = x - t - more
        t += more
// no `{t:.3f}` in either prototype: print thousandths by hand
let th = Int(t * 1000.0 + 0.5)
let frac = th % 1000
let pad = if frac < 10: "00" elif frac < 100: "0" else: ""
print("{th / 1000}.{pad}{frac}")
