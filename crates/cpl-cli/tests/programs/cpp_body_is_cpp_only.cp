// A file with an `@cpp` fn traces like any other while the run doesn't
// call it; only the C++ build runs that body, as `cpl run` does by itself.
fn pc(a: Int) -> Int = @cpp("return __builtin_popcountll(a);")
let n = 7
print(n + 1)
if n > 100:
    print(pc(n))
