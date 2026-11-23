// expect: ok, output "14"
// rule: 04.2.28
// `Int(x)` dispatches through `impl Into<Int>` inside a generic fn bounded by `T: Into<Int>` (decision 390).
struct Mask:
    bits: Int
impl Into<Int> for Mask:
    fn into(self) -> Int = self.bits
fn total<T: Into<Int>>(xs: List<T>) -> Int:
    var s = 0
    for x in xs: s += Int(x)
    s
print(total([Mask(bits: 5), Mask(bits: 9)]))
