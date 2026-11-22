// A `[K]T` parameter takes a list or its slice even when `K` is not named
// again (R04b.3.1).
fn first_of<K>(a: [K]Int) -> Int = a.first!
fn size<K>(a: [K]Int) -> Int = a.len
let s = [1, 2, 3, 4, 9]
print(first_of(s), first_of(s[1..3]), size(s), size(s[2..]))
