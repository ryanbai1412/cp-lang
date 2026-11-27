// expect: E0606
// rule: R08.1.5
// `?T`'s `==` is its impl's, which needs `T: Eq`, and a fn value has no `Eq`. (`== None` is a
// presence test and needs none, R02.4.5.)
fn mk() -> ?(Int -> Int) = None
let c = mk()
if c == mk(): print(1) else: print(0) // <- here
