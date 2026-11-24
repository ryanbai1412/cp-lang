// IOI 2025 festival's shape: the grader passes lists, the fn takes them over
// its own domain C and gives coupons of C back.
@grader
fn max_coupons(a: 0..=1e9, p: [C]1..=1e9, t: [C]1..=4) -> List<C> =
    C.filter(c => p[c] <= a and t[c] > 1).sorted(by: c => p[c])
