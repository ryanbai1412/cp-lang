// expect: E0302 at 4
// rule: 07.3.5
// `.rev` is not `Indexed` (R07.3.5), so it can't pass where a bound reads `s[k]` by key; generic code that wants position asks for less (rebuild round 29 H02).
fn size<A: Indexed>(s: A) -> Int = s.len
let a = [1, 2, 3]
print(size(a.rev))
