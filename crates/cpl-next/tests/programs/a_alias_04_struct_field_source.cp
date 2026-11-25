// expect: ok, output "1 5"
// rule: same
// Same with the list inside a struct global, passed as a field.
struct S:
    var xs: List<Int>
var s = S([1, 2, 3, 4])
fn f(a: List<Int>) -> Int:
    s.xs.push(5)
    a[0] * 1
print(f(s.xs), s.xs.len)
