var m: HashMap<Int, Int> = []
var x = 123456789
for i in 0..500_000:
    x = (x * 1103515245 + 12345) % 2147483647
    let k = x % 400_000
    m[k] = (m.get(k) ?? 0) + 1
var sum = 0
for (k, v) in m:
    sum += k * v
print(sum)
