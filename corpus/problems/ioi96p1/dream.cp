// IOI 1996 A Game: players alternately take a number from either end; both play optimally.
// The player to move on a range gets its sum minus the best the other gets on what remains.
domain I = [read Int]
let a = read [I]Int
let pre = Prefix(a)
@memo(array)
fn best(r: Range<I>) -> Int:
    if r.len == 1: return a[r.first!]
    pre.sum(r) - min(best(r.drop(1)), best(r.drop_last(1)))
let first = best(I.all)
print(first, a.sum() - first)
