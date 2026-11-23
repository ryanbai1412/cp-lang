// expect: ok, output "YES YES"
// input: "998244353"
use std.ntt.{FastProduct}
let m: 2..=2 * 10 ** 9 = read()
type Z = Mod<m>
type W = Mod<1000000007>
let a: List<Z> = [Z(i + 1) for i in 0..100]
let b: List<Z> = [Z(2 * i + 1) for i in 0..100]
var brute = List.filled(199, Z(0))
for i in 0..100:
    for j in 0..100: brute[i + j] += a[i] * b[j]
let w: List<W> = [W(i) for i in 0..100]
print(yes(Z.product(a, b) == Some(brute)), yes(W.product(w, w) == None))
