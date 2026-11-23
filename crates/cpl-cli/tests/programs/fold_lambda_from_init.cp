domain Pos = 0..4
fn later(a: ?Pos, b: ?Pos) -> ?Pos = b ?? a
let root: ?Pos = Pos.fold((r, p) => later(r, p), None)
let named: ?Pos = Pos.fold((r, p) => later(r, p), init: None)
let xs = [3, 1, 2]
let best: ?Int = xs.fold((m, x) => if x > 1: Some(x) else: m, None)
print(Int(root!), Int(named!), best!)
