// expect: ok, output "5 2\n1 9"
// rule: 02.6.2
// A range type folds by `max` and `min`: its ends are the identity, so the fold stays in range.
let xs: List<1..=9> = [3, 5, 2]
print(xs.fold(max), xs.fold(min))
let e: List<1..=9> = []
print(e.fold(max), e.fold(min))
