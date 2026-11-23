// expect: ok, output "YES"
// rule: 07.10.1
// A record literal compared with a typed value takes that value's field types, so an empty
// list field needs no annotation.
fn split(a: List<Int>) -> (q: List<Int>, r: List<Int>) = (q: a, r: [])
print(yes(split([1, 2]) == (q: [1, 2], r: [])))
