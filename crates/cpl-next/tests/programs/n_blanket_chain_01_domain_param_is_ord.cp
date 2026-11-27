// expect: ok, output "1 2 YES"
// rule: R08.1.4
// A blanket impl's bound can be met by another blanket: `impl<D: Domain + Eq> Ord for D` holds
// for a `D: Domain` because `impl<D: Domain> Eq for D` gives it `Eq`, so `<` works on it.
fn lo<D: Domain>(u: D, v: D) -> D = if u < v: u else: v
domain V = 0..4
print(lo(V(2), V(1)).ord, lo(V(2), V(3)).ord, if V(1) < V(2): "YES" else: "NO")
