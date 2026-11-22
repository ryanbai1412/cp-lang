// ioi96p1 A Game: players take a number from either end; both play best. The mover's best total
// on a range is its sum minus the opponent's best on what is left.
let n = read<1..=100>()
domain I = [n]
let a = read<[I]0..=200>()
@memo(array)
fn best(r: I.ranges) -> Int:
    if r.first! == r.last!:
        return a[r.first!]
    a[r].sum() - min(best(r.drop(1)), best(r.drop_last(1)))
let first = best(I.range)
print(first, a.sum() - first)
