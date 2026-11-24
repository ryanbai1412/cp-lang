// expect: ok, output "1 2 5 6"
// rule: 05.7.9 05.10.4
// A lambda that takes its parameter as `var` sinks it, and one that stores a `.clone()` keeps a copy: both may store it in an outer `var`, whichever argument binds the generic.
fn take<M>(undo: M -> (), v: M):
    undo(v)
fn drive<M>(mark: () -> M, undo: M -> ()):
    let m = mark()
    undo(m)
var a = List.filled(3, 0)
var b = List.filled(3, 0)
take((var m) => a = m, [1, 2])
drive(mark: () => [5, 6], undo: m => b = m.clone())
print(a, b)
