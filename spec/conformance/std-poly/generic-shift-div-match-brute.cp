// expect: ok, output "YES\nYES\nYES\nYES\nYES\nYES"
// `Poly<T>.shift` splits at a power of two and `div` goes through `inv` of the reversed divisor
// when its lead is a unit, so both are fast for a `FastProduct` `T`; this checks both against
// Horner and long division, over `Mod<P>` and over exact `Int`, past the size where they switch.
// Each long division leaves its dividend whole for the check that reads it next.
// copies: 4
use std.poly.{Poly}

type Z = Mod<998244353>

fn horner_shift<T: Div>(f: Poly<T>, c: T) -> Poly<T>:
    var acc = Poly<T>.zero
    for i in (0..f.len).rev: acc = acc * Poly([c, T.one]) + Poly([f.c[i]])
    acc.truncated(f.len)

fn long_div<T: Div + Sub>(a: Poly<T>, b: Poly<T>) -> Poly<T>:
    let db = b.deg()!
    let da = a.deg()!
    var r = a.c
    var q = List.filled(da - db + 1, T.zero)
    for i in (0..=da - db).rev:
        let f = r[i + db] / b.c[db]
        q[i] = f
        for j in 0..=db: r[i + j] = r[i + j] - f * b.c[j]
    Poly(q)

fn same<T: Div>(a: Poly<T>, b: Poly<T>) -> Bool:
    a.len == b.len and all(a.at(i) == b.at(i) for i in 0..a.len)

let fz = Poly<Z>([Z((i * 7919 + 13) % 1000) for i in 0..300])
let gz = Poly<Z>([Z((i * 104729 + 5) % 997 + 1) for i in 0..120])
print(yes(same(fz.shift(Z(3)), horner_shift(fz, Z(3)))))
print(yes(same(fz / gz, long_div(fz, gz))))

let fi = Poly<Int>([(i * 31 + 7) % 5 - 2 for i in 0..100])
let gi = Poly<Int>([(i * 17 + 3) % 3 - 1 for i in 0..40] + [1])
let hi = Poly<Int>([(i * 31 + 7) % 3 for i in 0..40])
print(yes(same(hi.shift(1), horner_shift(hi, 1))))
print(yes(same(fi / gi, long_div(fi, gi))))

let gi2 = Poly<Int>([1 for _ in 0..40] + [2])
let di = fi / gi2
print(yes(same(di, long_div(fi, gi2))))

let fs = Poly<Z>([5, 4, 3, 2, 1])
let gs = Poly<Z>([1, 1])
let ds = fs / gs
print(yes(same(ds, long_div(fs, gs))))
