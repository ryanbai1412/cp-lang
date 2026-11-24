// An empty list inside a printed value has nothing to show; printing it once must not ask its `impl Show` again forever (cf2089e's `[V]List<Z>`).
use std.mod.{Mod}
type Z = Mod<998_244_353>
domain V = 0..2
var dp: [V]List<Z> = []
dp[V(1)].push(Z(3))
print(dp)
let l: List<List<Z>> = [[], [Z(4), Z(5)]]
print(l)
print((Z(6), dp[V(0)]))
