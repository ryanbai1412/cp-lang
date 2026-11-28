// expect: ok, output "5 15"
// rule: contract 4: a lender finishes first
// A lender whose tail repairs a total; the body leaves early with `return`: the lender finishes first (decision: lender runs the rest).
struct Acc:
    var xs: List<Int>
    var total: Int
    fn at(mut self, i: Int) -> lends mut Int:
        var cell = xs[i]
        let before = cell
        lend cell
        xs[i] = cell
        total += cell - before
fn bump(mut a: Acc) -> Int:
    a.at(0) += 4
    a.xs[0]
var a = Acc([1, 10], 11)
let r = bump(mut a)
print(r, a.total)
