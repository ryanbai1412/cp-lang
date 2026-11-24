// expect: ok, output "0 1 0 3\n0 0 1 1\n0 0 1 1\n1 0 0"
// rule: 06.2.1
// Sibling closures share the variable they capture: one reads it (a copy
// it returns), one writes into it, one reassigns it, and each sees the
// others' changes, inside a fn, at top level and in a loop body.
fn drive<M>(mark: () -> M, add: Int -> (), undo: M -> (), reset: () -> ()):
    let m = mark()
    add(1)
    add(2)
    undo(m)
    add(0)
    reset()
    add(2)
fn read_then_write():
    var cnt = List.filled(3, 0)
    let m = (() => cnt.clone())()
    let add = (p: Int) => cnt[p] += 1
    add(1)
    print(cnt, m.len)
fn in_fn():
    var cnt = List.filled(3, 0)
    var resets = 0
    drive(mark: () => cnt.clone(), add: p => cnt[p] += 1, undo: (var m) => cnt = m, reset: () =>
        resets += 1
        cnt = List.filled(3, 0))
    print(cnt, resets)
read_then_write()
in_fn()
var cnt = List.filled(3, 0)
var resets = 0
drive(mark: () => cnt.clone(), add: p => cnt[p] += 1, undo: (var m) => cnt = m, reset: () =>
    resets += 1
    cnt = List.filled(3, 0))
print(cnt, resets)
fn run(add: Int -> (), undo: List<Int> -> (), mark: () -> List<Int>):
    let m = mark()
    add(1)
    undo(m)
    add(0)
for k in 0..1:
    var cnt = List.filled(3, 0)
    run(add: p => cnt[p] += 1, undo: m => cnt = m.clone(), mark: () => cnt.clone())
    print(cnt)
