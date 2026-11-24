// expect: ok, output "YES YES"
// Entries just under 2^60 in size take the flat pass and entries of 2^60 the loop; both give the
// loop's sums, so no sum lands on std's sentinel and reads as absent.
domain V = 0..2
fn square(x: Int) -> ?Int:
    let d: [V, V]?Int = [V, V] (_, _) => Some(x)
    (Mat(d, over: Semiring(max, +)) * Mat(d, over: Semiring(max, +))).get(1, 0)
let near = -((1 << 60) - 1)
let at = -(1 << 60)
print(yes(square(near) == Some(2 * near)), yes(square(at) == Some(2 * at)))
