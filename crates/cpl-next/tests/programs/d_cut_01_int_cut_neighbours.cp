// expect: ok, output "41 42\n-1 0"
// rule: R04.2.32 R04.3.2
// Every integer is a position of `Int`, so a `Cut<Int>` has a position on both sides, at 0 too.
let c = (0..100).bsearch(x => x >= 42)
print(c.l ?? -99, c.r ?? -99)
let z = (0..100).bsearch(x => x >= 0)
print(z.l ?? -99, z.r ?? -99)
