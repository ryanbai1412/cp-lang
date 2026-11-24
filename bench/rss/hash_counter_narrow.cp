// hash_counter with the key and count types stated (noi01p5's meet in the middle): the map
// stores each entry as two 32-bit ints.
type Sum = -2147483647..=2147483647
let n: 1..=4_000_000 = read()
var c: Counter<Sum, 0..=4_000_000> = Counter()
c.reserve(n)
var x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483647
    c.add(x - 1_000_000_000, 1)
var hits = 0
x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483647
    hits += c[if i % 2 == 0: x - 1_000_000_000 else: 1_147_483_647 + x % 1_000_000_000]
print(c.len, hits)
