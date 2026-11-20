// error: bind it first: `let x = a?`
fn pick(a: ?Int, use_it: Bool) -> ?Int:
    let v = if use_it: a? + 1 else: 0
    v * 2
print(pick(none, false) ?? -1)
