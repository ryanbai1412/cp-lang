// expect: ok, output "3 64 3 8"
// rule: 12.14.3
// A call of an `@intrinsic` fn is the table's primitive: `popcount` counts an `Int`'s two's complement and a `U64` as it is, and stays a fn value.
fn ones(x: Int) -> Int = @intrinsic("popcount")
let f = ones
print(popcount(7), ones(-1), popcount(U64(11)), f(255))
