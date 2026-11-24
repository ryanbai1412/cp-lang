// A Counter keyed by a one-field struct, churned: each `add` either finds
// its entry once or puts it, and a count back at 0 is erased.
struct Key(v: Int)
var c = Counter<Key>()
c.reserve(1_000_000)
var x = 987654321
for i in 0..4_000_000:
    x = (x * 1103515245 + 12345) % 2147483647
    c.add(Key(x % 1_000_000), if i % 4 == 3: -1 else: 1)
print(c.len)
