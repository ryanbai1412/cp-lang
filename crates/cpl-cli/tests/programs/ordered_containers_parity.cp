var st: Set<Int> = Set()
for x in [10, 20, 30, 40, 50]: st.add(x)
st.set(25, on: true)
st.set(25, on: true)
st.set(40, on: false)
st.set(40, on: false)

let sv = st.range(15..=35)
print(sv.first ?? -1, sv.last ?? -1, yes(sv.is_empty), [x for x in sv], [x for x in sv.rev])
let sopen = st.range(..25)
let sclose = st.range(25..)
print(sopen.first ?? -1, sopen.last ?? -1, [x for x in sopen], [x for x in sclose])
print(yes(st.range(26..30).is_empty), st.range(26..30).first ?? -1, st.range(26..30).last ?? -1)

var hs: HashSet<Int> = HashSet()
hs.set(7, on: true)
hs.set(9, on: true)
hs.set(7, on: false)
print(hs.len, yes(7 in hs), yes(9 in hs))

var mp: Map<Int, Int> = Map()
for k in [2, 5, 8, 12]: mp[k] = k * 10
let mv = mp.range(3..=10)
let (fk, fv) = mv.first!
let (lk, lv) = mv.last!
print(fk, fv, lk, lv, yes(mv.is_empty), [k + v for (k, v) in mv], [k for (k, _) in mv.rev])
let (pk, _) = mp.range(..5).last!
let (_, nv) = mp.range(8..).first!
print(yes(mp.range(6..8).is_empty), pk, nv)

var ss = SortedSet<Int>.of([10, 20, 30, 40])
ss.set(25, on: true)
ss.set(30, on: false)
print(ss.len, ss.first_true(ok: x => x >= 22) ?? -1, ss.last_true(ok: x => x <= 22) ?? -1)
print(ss.first_true(x => x > 100) ?? -1, ss.last_true(x => x < 0) ?? -1)

var sb = SortedBag<Int>.of([5, 5, 10, 20, 20, 20])
print(sb.first_true(x => x >= 10) ?? -1, sb.last_true(x => x <= 10) ?? -1, sb.range(5..=20).len)

domain D = 1..=6
var ds: Set<D> = Set()
for i in [1, 3, 5]: ds.set(D(i), on: true)
let r = D(2)..D(5)
print(ds.range(r).first!.ord, ds.range(r).last!.ord, [x.ord for x in ds.range(r)])
