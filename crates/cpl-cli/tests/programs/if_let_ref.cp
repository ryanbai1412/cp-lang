// An `if let` or `while let` payload nothing changes is read where it
// lives; one whose source changes stays what it was.
fn total(n: Int, end: ?List<Int>) -> Int:
    var s = 0
    for i in 0..n:
        if let e = end: s += e[i % e.len]
    s

var best: ?List<Int> = Some([1])

fn swap_best() -> Int:
    if let e = best:
        best = Some([2, 3, 4])
        return e.len
    0

fn drain(var stack: List<(dies: Int, val: Int)>) -> Int:
    var s = 0
    while let top = stack.last:
        stack.pop()
        stack.push((dies: top.dies - 1, val: top.val))
        if top.dies <= 1: stack.pop()
        s += top.val
    s

var rows: List<?List<Int>> = [Some([5, 6]), None, Some([7])]

fn bump(r: Int) -> Int:
    if let row = rows[r]:
        rows[r] = Some([0])
        return row[0]
    -1

print(total(5, Some([1, 2, 3])), total(3, None))
print(swap_best(), best!.len)
print(drain([(dies: 2, val: 10), (dies: 1, val: 3)]))
print(bump(0), bump(1), bump(2), rows[0]!.len)
