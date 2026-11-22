// A `@memo fn` below `@!multitest` belongs to its test case: the cache is empty at the start of each one.
// Every case asks for the same keys over a different array, so a cache kept from the first case would
// answer the later ones with the first case's sums.
let scale: 1..=3 = read()       // above the divider: read once, shared

@!multitest
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]0..=9 = read()

@memo
fn suffix(i: V) -> Int:
    if let j = i.next:
        return a[i] * scale + suffix(j)
    a[i] * scale

print(suffix(V.first), suffix(V.first))
