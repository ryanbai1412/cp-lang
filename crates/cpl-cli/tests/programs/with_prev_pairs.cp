use std.seq.{adjacent, with_prev}

let xs = [10, 20, 35]
for (a, b) in xs.adjacent():
    print(b - a)

for (p, x) in xs.with_prev():
    print(p ?? 0, x)

domain I = 0..3
var arr: [I]Int = 0
arr[I.at(0)!] = 2
arr[I.at(1)!] = 5
arr[I.at(2)!] = 9
print([b - a for (a, b) in arr.adjacent()])
print([(p ?? -1, x) for (p, x) in arr.with_prev()])
