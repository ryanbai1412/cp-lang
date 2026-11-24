// expect: E0606
// rule: 07.2.18
// `impl Indexed for Items<S: Indexed>` gives `Items<S>` nothing where `S` is not Indexed: `a.rev` is not, so neither is `a.rev.items`.
fn need<S: Indexed<D, T>, D, T>(s: S) -> Int = 0
let a = [0, 1, 2, 3, 4]
print(need(a.rev.items))  // <- here
