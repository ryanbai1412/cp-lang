// expect: ok, output "6\n15"
// rule: R10.1.1
// input: "2\n3\n1 2 3\n5\n1 2 3 4 5\n"
// A fn declared below the divider captures that case's names (R10.1.1).
@!multitest
let n: Int = read()
let a = read<[n]Int>()
fn total() -> Int:
    var s = 0
    for x in a: s += x
    s
print(total())
