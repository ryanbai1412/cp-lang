// expect: ok, output "12 4 24 9"
domain R = 0..3
domain C = 0..4
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 4 + Int(c)
fn count(x: Int) -> Int:
    let p = Prefix([R, C] (i, j) => Int(g[i, j] <= x))
    p.fold(.., ..)
fn sums(rows: Range<R>, cols: Range<C>) -> Int:
    let p = Prefix([R, C] (i, j) => g[i, j])
    p.fold(rows, cols)
print(count(11), count(3), sums(0..=1, 1..=3), sums(2..=2, 1..=1))
