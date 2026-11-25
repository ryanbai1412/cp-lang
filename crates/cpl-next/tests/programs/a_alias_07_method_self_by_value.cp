// expect: ok, output "1 5"
// rule: same as alias_01
// A method taking self by value while a closure passed in writes the receiver's global.
struct S:
    var xs: List<Int>
    fn first_after(self, k: () -> Int) -> Int:
        let n = k()
        xs[0] * 1 + n * 0
var s = S([1, 2, 3, 4])
fn go() -> Int:
    s.first_after(() => { s.xs.push(5); 0 })
print(go(), s.xs.len)
