// Repro: unqualified call to user `fn apply(...)` taking a `List` / array collides
// with `std::apply` via C++ ADL (Argument-Dependent Lookup) and fails g++ compilation!
// Discovered in: proto/ports/scout_batch_11/noi20p1.cp (g++ compile failure)
// Cause: `best` has C++ type `std::vector<ll>` (in `namespace std`), so unqualified
// `apply(best, mat)` finds `std::apply(F&&, Tuple&&)` via ADL, which beats
// `prog::apply(std::vector<ll>, ...)` on non-const lvalue arguments.
// Fix: qualify user function calls (`prog::apply(...)`) or mangle/prefix user identifiers.

let n: Int = read()
domain D = 0..n
var vec: [D]Int = 0
var mat: [D, D]Int = 0

fn apply(v: [D]Int, m: [D, D]Int) -> [D]Int:
    var out: [D]Int = 0
    for i in D:
        out[i] = v[i] + m[i, i]
    out

vec = apply(vec, mat)
print(vec[D(0)])
