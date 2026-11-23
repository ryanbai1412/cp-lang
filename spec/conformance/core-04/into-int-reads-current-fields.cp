// expect: ok, output "3 4 1 2 3 4"
// rule: 04.2.28
// `Int(b)` calls `into` on `b` as it is now; the earlier result is a plain `Int`.
struct Big:
    xs: List<Int>
impl Into<Int> for Big:
    fn into(self) -> Int = self.xs.len
var b = Big(xs: [1, 2, 3])
let n = Int(b)
b.xs.push(4)
print(n, Int(b), b.xs)
