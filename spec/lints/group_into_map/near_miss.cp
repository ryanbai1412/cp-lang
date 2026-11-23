// A loop that also reads the map, or pushes under a condition, is doing more than grouping.
let n: 1..=200000 = read()
let a: [0..n]Int = read()
var pos: HashMap<Int, List<Int>> = HashMap()
for i in 0..n:
    if a[i] > 0:
        (pos[a[i] % 3] ??= []).push(i)
var seen: HashMap<Int, List<Int>> = HashMap()
for i in 0..n:
    print(seen.get(a[i])?.len ?? 0)
    (seen[a[i]] ??= []).push(i)
print(pos.get(0)?.len ?? 0)
