// A fn whose signature does not parse and a statement the grammar has no form for: the
// units around them type and lower.
fn first(a: Int) -> Int:
    a * 3

fn second(a: Int, ) -> : // injected
    a

let k = first(2)
let m = k + * 3 // injected
print(k)
fn third() -> Int:
    first(k)
print(third())
