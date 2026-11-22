// expect: ok, output "-1 9"
// rule: 04.2.14
domain V = 0..10
fn at(i: Int) -> Int:
    if let V(j) = i - 10: return Int(j)
    -1
print(at(20), at(19))
