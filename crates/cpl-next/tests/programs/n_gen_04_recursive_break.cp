// expect: ok, output "1\n2\n3\n4"
// rule: R07.6.2
// A recursive generator held as a value walks lazily, and `break` ends the walk.
fn nat_from(n: Int) -> yields Int:
    yield n
    for x in nat_from(n + 1): yield x
let g = nat_from(1)
for x in g:
    if x > 4: break
    print(x)
