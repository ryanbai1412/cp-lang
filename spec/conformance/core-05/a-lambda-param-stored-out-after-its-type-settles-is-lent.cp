// expect: E0603
// rule: 05.7.9 05.10.4
// A lambda's parameter is lent when the call had already bound its generic from an earlier argument.
fn drive<M>(mark: () -> M, undo: M -> ()):
    let m = mark()
    undo(m)
var cnt = List.filled(3, 0)
drive(mark: () => [1, 2], undo: m => cnt = m) // <- here
print(cnt)
