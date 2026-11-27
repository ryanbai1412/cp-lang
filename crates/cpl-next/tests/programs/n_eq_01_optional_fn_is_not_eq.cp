// expect: E0606
// rule: R08.1.5
// `?T`'s `==` is its impl's, which needs `T: Eq`, and a fn value has no `Eq`.
fn mk() -> ?(Int -> Int) = None
let c = mk()
if c == None: print(1) else: print(0) // <- here
