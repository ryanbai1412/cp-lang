var m: HashMap<Int, Int> = []
var x = 987654321
for i in 0..200_000:
    x = (x * 1103515245 + 12345) % 2147483647
    m[x % 400_000] = i + 1
var sum = 0
for _ in 0..2_000_000:
    x = (x * 1103515245 + 12345) % 2147483647
    if let v = m.get(x % 400_000): sum += v
print(sum)
