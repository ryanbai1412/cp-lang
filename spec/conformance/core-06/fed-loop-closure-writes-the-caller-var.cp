// expect: ok, output "603 72"
// rule: 06.3.1
// A loop a generator feeds may call a nested fn that writes the caller's own `var`s: the generator
// still pushes into the loop, and the loop's `return` stops it.
fn upto(xs: List<Int>) -> yields Int:
    for x in xs: yield x

fn walk() -> Int:
    let xs = [1, 2, 3, 4]
    var seen = 0
    var log = List<Int>()
    fn grow(x: Int):
        seen += x
        log.push(seen)
    for x in upto(xs):
        grow(x)
        if seen > 5: return seen * 100 + log.len
    -1

fn count() -> Int:
    var c = 0
    fn bump(): c += 1
    for _ in 0..5: bump()
    for x in upto([1, 2, 3]):
        bump()
        if c > 6: return c * 10 + x
    c

print(walk(), count())
