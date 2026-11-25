// expect: ok, output "7"
// rule: R08
// An inherent impl block (no trait): methods added to a struct.
struct P:
    x: Int
impl P:
    fn twice(self) -> Int = x * 2
print(P(3).twice() + 1)
