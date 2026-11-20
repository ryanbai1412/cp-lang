// Deprecated spelling, still built so older ports keep working: a bare `@multitest` on the first statement
// of the test case divides the file the way `@!multitest` does.
var seen = 0
@multitest
let n: 0..=9 = read()
seen += 1
if n == 0:
    return
print(n, seen)
