// `Int` is symmetric, so LLONG_MIN is the `?Int` niche and no key; the hash table must still keep the
// most negative `Int`, one above it, as an ordinary key. Checks it with backward-shift deletion,
// re-insertion, and a Counter<Int>'s key count under ASan/UBSan.
let sentinel = -9223372036854775807
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
var c = Counter([sentinel, 0])
c[sentinel] += 2
c[0] += 1
c[sentinel] -= 3
let d1 = c.len
c[0] -= 2
print(yes(has_s1), val_s, yes(has_s2), m.len, d1, c.len)
