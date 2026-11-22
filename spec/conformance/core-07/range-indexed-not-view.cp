// expect: ok, output "3 9 9"
// rule: 07.2.5 07.2.15
fn span() -> Range<Int> = 2..5
let r = span()
print(r.len, r.sum(), [0, 1, 2, 3, 4, 5][r].sum())
