// expect: L0508 at 1
fn hit(p: ?Int, v: Int) -> Bool = p != None and v == p!
print(hit(None, 1))
