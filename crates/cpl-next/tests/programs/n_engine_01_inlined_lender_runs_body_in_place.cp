// expect: ok, output "24 12"
// input: "5\n"
// rule: contract 4: a lender finishes first
// The engine compiles a small lender into its caller and runs the region body where the
// lender lends; the lender's tail still runs after it, and a lender lending through another
// lends the inner place.
struct Acc:
    var xs: List<Int>
    var total: Int
    fn at(mut self, i: Int) -> lends mut Int:
        var cell = xs[i]
        let before = cell
        lend cell
        xs[i] = cell
        total += cell - before
let n: 1..=10 = read()
var a = Acc([1, 2, 3], 6)
var grid: [0..n]Int = 0
for i in 0..3:
    a.at(i) += i + 1
    grid[i] += a.xs[i]
print(grid[0] + grid[1] + grid[2] + a.total, a.total)
