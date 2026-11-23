let n: 1..=200000 = read()
let a: [0..n]Int = read()
var pos: HashMap<Int, List<Int>> = HashMap()
for i in 0..n:
    (pos[a[i] % 3] ??= []).push(i)
print(pos.get(0)?.len ?? 0)
