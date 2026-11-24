// expect: E0603
// rule: 05.7.9 05.10.4
// A lambda's parameter is lent whether or not the call had bound its generic when the lambda was checked: here `M` binds from a later argument.
fn take<M>(undo: M -> (), v: M):
    undo(v)
var cnt = List.filled(3, 0)
take(m => cnt = m, [1, 2]) // <- here
print(cnt)
