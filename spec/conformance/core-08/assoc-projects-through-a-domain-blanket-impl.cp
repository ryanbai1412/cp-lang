// expect: ok, output "2.0000000000\n3"
// rule: 08.1.9
// `D.Found` projects through the impl a type gets: the domain blanket `impl<D: Domain>` for a position, the F64 impl for a float (decision 456).
domain V = 0..3

trait Pick:
    type Found
    fn pick(x: Self) -> Self.Found

impl<D: Domain> Pick for D:
    type Found = Int
    fn pick(x: D) -> Int = x.ord

impl Pick for F64:
    type Found = ?F64
    fn pick(x: F64) -> ?F64 = Some(x)

fn go<D: Pick>(x: D) -> D.Found = D.pick(x)

print(go(2.0)!)
print(go(V(2)) + 1)
