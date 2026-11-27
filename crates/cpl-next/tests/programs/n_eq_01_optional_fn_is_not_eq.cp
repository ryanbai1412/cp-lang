// expect: E0606
// rule: R08.1.5
// `?T`'s `==` is its impl's, which needs `T: Eq`, and a fn value has no `Eq`. Comparing with
// the `None` value is a presence test and needs none (R02.4.5, n_eq_02); `mk()` is not that value.
fn mk() -> ?(Int -> Int) = None
let c = mk()
if c == mk(): print(1) else: print(0) // <- here
