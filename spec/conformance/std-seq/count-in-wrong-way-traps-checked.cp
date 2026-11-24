// expect: R0401 at 4 (checked)
// `count_in` wants sorted input: a decreasing list makes its search wrong-way, which a checked build traps on in both engines.
let a = [1, 3, 3, 5, 8]
let e: List<Int> = []
print(a.count_in(3..6), a.count_in(4..5), a.count_in(-100..100), e.count_in(0..9), [8, 5, 3].count_in(4..9, by: x => x))
