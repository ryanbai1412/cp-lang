// expect: E0606
// rule: 08.2.2
fn z<T: Zero>(x: T) -> T = T.zero
print(z((1, 2))) // <- here
