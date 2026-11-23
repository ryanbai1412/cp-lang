var f: [0..=20]Int = 0
f[1] = 1
for i in 2..=20:
    f[i] = f[i - 1] + f[i - 2]
var seen: List<Int> = []
for i in 0..3: seen.push(f[i * 5])
@!multitest
let k: 0..=20 = read()
print(f[k], seen.len)
