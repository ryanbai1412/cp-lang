// expect: E0608 at 4
// rule: 05.2.2
var xs = List(1, 2)
var keep: List<List<Int>> = []
for i in 0..2:
    keep.push(xs)
print(keep.len)
