// A traced loop fed by an `@cpp` fn runs on the VM like any fed loop; only
// the C++ build runs that body, so a run that never reaches it traces on.
fn bits(a: Int) -> yields Int = @cpp("std::vector<ll> r; for (int b = 0; b < 63; b++) if (a >> b & 1) r.push_back(b); return r;")

fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i

let n = 7
var total = 0
for x in evens(n):
    total += x
print(total)
if n > 100:
    for b in bits(n):
        print(b)
