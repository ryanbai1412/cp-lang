// expect: ok, output "11400714819323198485 18446744073709551615 7\n2654435769"
// rule: R02.1
// An integer literal in a `U64` slot is that `U64`, past `Int`'s top too.
fn mix(x: U64) -> U64:
    var z: U64 = x + 0x9e3779b97f4a7c15
    z = z ^ 0
    z
let top: U64 = 18446744073709551615
var w: U64 = 3
w = w + 4
print(mix(0), top, w)
print(mix(0) >> 32)
