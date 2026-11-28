// expect: R0202
// A trap in a region body the engine runs inside an inlined lender reports the caller's
// line and call stack, as the reference interpreter does.
struct Cells:
    var xs: List<Int>
    fn at(mut self, i: Int) -> lends mut Int:
        var cell = xs[i]
        lend cell
        xs[i] = cell
fn bump(mut c: Cells, k: Int):
    for i in 0..3:
        c.at(i) += k  // <- here
var c = Cells([9_223_372_036_854_775_800, 1, 2])
bump(mut c, 3)
bump(mut c, 5)
print(c.xs[0])
