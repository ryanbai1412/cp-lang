// expect: ok, output "0.5000000000 2 7"
// A table keys a Map in both engines: a 2-D board of optional enum cells inside a tuple, and a
// 1-D array. The C++ hash walks the cells, so equal tables find the same entry.
// copies: 1
domain R = 0..2
domain C = 0..3
enum P = J | D
var memo: Map<([R, C]?P, P), F64> = Map()
var b: [R, C]?P = None
b[R(0), C(1)] = Some(J)
memo[(b, J)] = 0.5
var same: [R, C]?P = None
same[R(0), C(1)] = Some(J)
var seen: Map<[R]Int, Int> = Map()
let row: [R]Int = 3
seen[row] = 7
print(memo.get((same, J)) ?? 0.0, memo.len + (if (b, D) in memo: 10 else: 1), seen[[R] _ => 3])
