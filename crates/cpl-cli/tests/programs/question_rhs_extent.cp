fn half(o: ?Int) -> ?Int:
    let v = o? / 2
    return v
fn plus(o: ?Int) -> ?Int:
    return o? + 1
let n = read<Int>()
let o: ?Int = if n > 0: Some(n) else: None
var w: ?Int = None
w = (o? * 3)
print(half(o) ?? -1, plus(None) ?? -1, w ?? -1, (o? + 1) ?? -1)
print(half(None) ?? -1, plus(o) ?? -1)
