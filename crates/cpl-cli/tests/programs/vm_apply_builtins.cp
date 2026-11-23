// The VM evaluates builtins, arithmetic and negation on values it holds in
// registers: wide and float arithmetic, `in`, min/max, rounding, sorting.
var seen = Set<Int>()
var m = Map<Int, Int>()
for i in 0..20:
    seen.add(i * 3 % 7)
    m[i % 5] = i
var hits = 0
var big = I128(1)
var f = 0.0
for i in 0..20:
    if i in seen: hits += 1
    if i in m: hits += 10
    big = big * I128(3) + I128(i) - I128(1) << 1 >> 1
    f = f + F64(i) / 4.0 - -0.5
print(hits, big % I128(1000000007), floor(f), ceil(f / 3.0), round(f / 7.0))
var lo = "zz"
var hi = 1.0
for w in ["pear", "fig", "kiwi"]:
    lo = min(lo, w)
    hi = max(hi, -F64(w.len))
print(lo, hi, abs(-hi), [3, 1, 2].sorted(desc: true), [i * i for i in 0..4].rev)
