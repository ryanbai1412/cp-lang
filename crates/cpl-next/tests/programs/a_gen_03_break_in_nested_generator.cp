// expect: ok, output "3 1"
// rule: contract 4 regions
// break inside a nested generator loop leaves only the inner loop; the outer generator goes on.
fn up(n: Int) -> yields Int:
    for i in 0..n: yield i
var hits = 0
var last = 0
for a in up(3):
    for b in up(3):
        if b == 1: break
        last = b + 1
    hits += 1
print(hits, last)
