// CpHashMap<ll, ll> uses LLONG_MIN as empty sentinel with a side slot; verify sentinel keys,
// backward-shift deletion, re-insertion, and Counter<Int> distinct counts under ASan/UBSan.
let sentinel = -9223372036854775807 - 1
var m: HashMap<Int, Int> = []
m[sentinel] = 42
for i in 0..200:
    m[i * 17 - 100] = i
for i in 0..100:
    m.remove(i * 17 - 100)
let has_s1 = sentinel in m
let val_s = m[sentinel]
m.remove(sentinel)
let has_s2 = sentinel in m
var c = counts([sentinel, 0])
c[sentinel] += 2
c[0] += 1
c[sentinel] -= 3
let d1 = c.distinct
c[0] -= 2
print(has_s1, val_s, has_s2, m.len, d1, c.distinct)
