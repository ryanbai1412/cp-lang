let n: Int = read()
var s = 0
for i in 0..8 * n:
    let x: ?Int = if i % 3 == 0: None else: Some(i)
    if x.is_some_and(v => v % 2 == 0): s += 1 // timed: opt.is_some_and
    if x.is_none_or(v => v % 5 == 0): s += 1 // timed: opt.is_none_or
    s += x.and_then(v => if v % 7 == 0: Some(v) else: None) ?? 0 // timed: opt.and_then
print(s)
