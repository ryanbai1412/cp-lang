// expect: ok, output "3"
// rule: 04.4.30
// `S<0..10>` is an anonymous domain when `S` uses its parameter as a domain, here only
// through `D.nth`; its positions are the Ints in the range.
struct S<D>(base: Int):
    fn at(self, i: Int) -> D = D.nth(base + i)
let s = S<0..10>(1)
print(s.at(2))
