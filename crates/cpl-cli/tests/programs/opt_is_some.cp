// std.opt's `is_some` and `is_none`, as getters and as calls, on a held value and on None, in both engines.
let n: Int = read()
let x: ?Int = if n > 0: Some(n) else: None
let y: ?Int = None
print(yes(x.is_some), yes(x.is_none), yes(y.is_some), yes(y.is_none))
print(yes(x.is_some()), yes(y.is_none()))
