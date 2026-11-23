let n: 1..=200000 = read()
let a: [0..n]1..=200000 = read()
var bit: [0..=200000]Int = 0
fn add(i0: Int):
    var i = i0
    while i <= n:  // expect: hand_rolled_range_tree
        bit[i] += 1
        i += i & -i
fn query(i0: Int) -> Int:
    var i = i0
    var s = 0
    while i > 0:  // expect: hand_rolled_range_tree
        s += bit[i]
        i -= i & -i
    return s
var inv = 0
for i in 0..n:
    inv += i - query(a[i])
    add(a[i])
print(inv)
