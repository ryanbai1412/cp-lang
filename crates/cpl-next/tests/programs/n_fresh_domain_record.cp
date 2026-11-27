// expect: ok, output "2 YES NO 1"
// rule: R02.9.3 R02.9.4
domain V = 0..4
fn halves() -> (domain C, of: [V]C):
    domain K = 0..2
    (C: K, of: [V] v => K(Int(v) / 2))
let (domain H, of) = halves()
print(H.count, yes(of[V(0)] == of[V(1)]), yes(of[V(1)] == of[V(2)]), Int(of[V(3)]))
