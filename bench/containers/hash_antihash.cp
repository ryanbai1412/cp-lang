// Adversarial anti-hash workload: multiples of libstdc++ prime 126271 and power-of-two stride 64.
var m: HashMap<Int, Int> = []
for i in 1..=150_000:
    m[i * 126271] = i
    m[i * 64] = i + 1
var sum = 0
for i in 1..=150_000:
    sum += (m.get(i * 126271) ?? 0) + (m.get(i * 64) ?? 0)
print(sum)
