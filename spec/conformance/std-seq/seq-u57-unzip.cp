// expect: ok, output "1 3 2.5000000000 4.5000000000 0"
// Acceptance for seq/U57 (split a list of pairs), handbook/product/std/sequences-usecases.md: `unzip` keeps each half's own type.
let (xs, ys) = [(1, 2.5), (3, 4.5)].unzip()
let (none_x, _) = [(1, 'a')].drop(1).unzip()
print(xs, ys, none_x.len)
