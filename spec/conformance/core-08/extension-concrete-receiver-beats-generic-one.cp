// expect: ok, output "3 2"
// rule: 08.1.4
// A fn whose `self` is `Range<Int>` is more specific than std's `first_true<D>(self: Range<D>)`: it takes the integer range, and a domain's range still calls std's.
fn first_true(self: Range<Int>, ok: Int -> Bool) -> Int:
    for x in self:
        if ok(x): return x
    -1
domain V = 0..10
print((0..10).first_true(x => x >= 3), (V(0)..V(9)).first_true(x => x.ord >= 2)!.ord)
