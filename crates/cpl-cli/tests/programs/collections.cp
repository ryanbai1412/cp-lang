// Lists, sets and maps. Sets and maps keep their keys in order, so
// what a program prints never depends on a hash.
let n: Int = read()
var xs: List<Int> = []
var seen: Set<Int> = []
var tally: Map<Int, Int> = []
for i in 0..n:
    let x: Int = read()
    xs.push(x)
    if seen.insert(x):
        tally[x] = 0
    tally[x] += 1
print(xs, xs.len, xs.first, xs.last)
print(seen, seen.len, seen.min(), seen.max())
print(tally)
for (k, v) in tally:
    print("key", k, "seen", v)
print(tally.get(3), tally.get(99), tally.contains(4), seen.contains(99))
xs.sort()
print(xs)
xs.reverse()
print(xs, xs.sorted(), xs.reversed())
xs.sort(by: x => abs(x - 4))
print(xs)
print(xs.pop(), xs.pop(), xs.len)
print(seen.remove(3), seen.remove(3), seen)
print(tally.remove(1), tally.len)
var stack: List<Int> = [1, 2, 3]
while stack.len > 0:
    let top = stack.pop() ?? -1
    print(top)
print(stack.pop(), stack.is_empty)
var pairs: List<(Int, Str)> = []
pairs.push((3, "c"))
pairs.push((1, "z"))
pairs.push((3, "a"))
pairs.sort()
print(pairs)
for (num, name) in pairs:
    print(name, num)
var nested: List<List<Int>> = []
for i in 0..3:
    var row: List<Int> = []
    for j in 0..=i:
        row.push(i * j)
    nested.push(row)
print(nested)
xs.clear()
print(xs.len, xs)
