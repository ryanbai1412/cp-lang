// expect: ok, output "100 12"
// rule: 08.2.4
// A `const` parameter is an `Int` in the body, so it takes part in
// arithmetic like any other value (found porting local-lib's Fixed<D>).
struct Fixed<const D: Int>(v: Int):
    fn scale(self) -> Int = 10 ** (D - 1)
fn shift<const K: Int>(x: Int) -> Int = x * K - K
print(Fixed<3>(5).scale(), shift<3>(5))
