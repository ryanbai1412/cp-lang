// expect: ok, output "6 3\n1 4 7\n10\n14\n14"
// rule: 07.1.3
// `take(k)` after `map` and `filter` on an infinite generator still stops it: only the values the
// adapters need are pulled, in a fold, a `collect` and a `for` alike.
var pulled = 0
fn nat() -> yields Int:
    var i = 0
    while true:
        pulled += 1
        yield i
        i += 1
print(nat().map(x => x * 2).take(3).sum(), pulled)
print(nat().filter(x => x % 3 == 0).map(x => x + 1).take(3).collect())
print(pulled)
var t = 0
for x in nat().map(x => x * x).take(4): t += x
print(t)
print(pulled)
