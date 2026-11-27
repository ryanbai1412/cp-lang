// expect: E0302
// rule: R08.2.5
// A field holding a fn value has no type parameters: type arguments written on it are an error.
struct Inc:
    f: Int -> Int
let b = Inc(f: x => x + 1)
print(b.f<Int>(1)) // <- here
