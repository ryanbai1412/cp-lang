// expect: ok, output "ok"
// input: "1 300\n"
// model: Fenwick
// Fenwick<V, Int> against a plain array: seeded random adds, each followed by a random range
// fold, prefix and point read. Reads `seed n`; prints the first op that differs.
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..37
var seed = seed0
var f = Fenwick<V, Int>()
var a = List.filled(37, 0)
var bad = ""
for k in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let (i, x) = (seed % 37, seed / 37 % 21 - 10)
    f.add(V(i), x)
    a[i] += x
    let lo = seed / 1000 % 37
    let hi = lo + seed / 100000 % (38 - lo)
    let want = sum(a[j] for j in lo..hi)
    if f.fold(V.cuts.nth(lo)..V.cuts.nth(hi)) != want:
        bad = "op {k}: fold {lo}..{hi}"
    elif f.prefix(V.cuts.nth(hi)) != sum(a[j] for j in 0..hi):
        bad = "op {k}: prefix {hi}"
    elif f.get(V(i)) != a[i]:
        bad = "op {k}: get {i}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
