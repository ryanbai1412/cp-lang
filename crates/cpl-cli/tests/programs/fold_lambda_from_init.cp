domain Pos = 0..4
fn later(a: ?Pos, b: ?Pos) -> ?Pos = b ?? a
let root: ?Pos = Pos.fold_from(None, (r, p) => later(r, p))
let named: ?Pos = Pos.fold_from(None, (r, p) => later(r, p))
let xs = [3, 1, 2]
let best: ?Int = xs.fold_from(None, (m, x) => if x > 1: Some(x) else: m)
print(Int(root!), Int(named!), best!)
