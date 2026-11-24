// expect: E0204
// rule: 02.1.3
// There is no boxed function type; a fn value's type is its signature.
let f: Box<Int -> Int> = x => x // <- here
print(f(1))
