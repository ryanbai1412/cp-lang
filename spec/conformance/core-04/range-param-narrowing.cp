// expect: ok, output "5"
// rule: 04.4.24
// An `Int` argument to a range-typed parameter is a checked narrowing, so `odd - 1` goes into `odd: 0..=100` (rebuild 8, H-RANGE-ARG-NARROW).
fn f(odd: 0..=100) -> Int:
    if odd == 0: return 0
    return 1 + f(odd - 1)
print(f(5))
