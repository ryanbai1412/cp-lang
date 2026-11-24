// expect: ok, output "NO 7\nYES NO"
// rule: 05.3.3
// A `Bool` element is a place like any other, so `mut lz[p]` lends it even where the element is packed and has no address of its own: the callee's write lands in the element, through a generic fn too.
fn compose_onto<F>(mut old: F, new: F, compose: (F, F) -> F):
    old = compose(new, old)
fn put<F>(mut lz: List<F>, p: Int, f: F, compose: (F, F) -> F):
    compose_onto(mut lz[p], f, compose)
var tags = [false, true, false]
put(mut tags, 1, true, (x, y) => x != y)
var ints = [1, 2, 3]
put(mut ints, 1, 5, (x, y) => x + y)
print(yes(tags[1]), ints[1])
fn flip(mut b: Bool): b = not b
var at = 0
fn next() -> Int:
    at += 1
    at - 1
flip(mut tags[next()])
print(yes(tags[0]), yes(tags[1]))
