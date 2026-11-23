// cf1991D Prime XOR Coloring: u ^ v prime and u = v mod 4 would make u ^ v a multiple of 4,
// so v mod 4 is a proper colouring; 1, 3, 4, 6 are pairwise prime-xor, so n >= 6 needs 4.
// Small n follow the samples.
use std.math.{is_prime}
@!multitest
let n: 1..=2e5 = read()
domain V = 1..=n
let small = [1, 2, 2, 3, 3]
print(if n >= 6: 4 else: small[n - 1])
print(V.map(v => if n >= 6: Int(v) % 4 + 1 else: small[v - 1]))

@check
fn ok() -> Bool:
    let n: 1..=2e5 = read()
    let k = read_output<Int>()
    let need = if n >= 6: 4 else: [1, 2, 2, 3, 3][n - 1]
    domain V = 1..=n
    let c = read_output<[V]1..=k>()
    k == need and not V.any(u => V.after(u).any(v => c[u] == c[v] and is_prime(Int(u) ^ Int(v))))
