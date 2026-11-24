// expect: ok, output "1 3 3"
// rule: 08.1.4
// `self: Range<D>` doesn't take a list, so a list's `bsearch` is `[D]T`'s, whichever way the list was typed.
fn first_big(xs: List<Int>) -> Int = xs.bsearch(x => x > 1)
let ys = [0, 1, 1, 5]
var zs: List<Int> = []
zs.push(1)
zs.push(2)
zs.push(3)
zs.push(4)
print(first_big([1, 2, 3]), ys.bsearch(y => y > 0) + 2, zs.bsearch(z => z > 3))
