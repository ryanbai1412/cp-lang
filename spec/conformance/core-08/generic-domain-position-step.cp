// expect: ok, output "YES YES"
// rule: 04.3.3
// A position of a generic domain moves by an integer like any position (`v + 1`), with no `R: Add` bound; grid `up`/`step` and search's `split_best` need it.
fn below<R>(p: R) -> ?R = R.at(p + 1)
domain R = 0..3
print(yes(below(R(1)) == Some(R(2))), yes(below(R(2)) == None))
