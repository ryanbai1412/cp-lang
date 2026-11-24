// 300k Str inserts into a HashMap grown from empty, then 300k lookups, half of them misses.
let n = 300_000
var m: HashMap<Str, Int> = []
var x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483647
    m["k{x}"] = i
var sum = 0
x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) % 2147483647
    let key = if i % 2 == 0: "k{x}" else: "m{x}"
    if let v = m.get(key): sum += v
print(m.len, sum)
