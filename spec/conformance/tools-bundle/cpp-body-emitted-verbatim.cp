// expect: ok
// rule: bundle.5
// tool: build -o -
// has: "return __builtin_popcountll(a);"
// An `@cpp("...")` body is C++ and goes into the judge file as written, so it can use what only C++ has.
fn pc(a: Int) -> Int = @cpp("return __builtin_popcountll(a);")
print(pc(7))
