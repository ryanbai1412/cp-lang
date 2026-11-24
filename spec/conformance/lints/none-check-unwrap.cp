// expect: L0508
fn hit(p: ?Int, v: Int) -> Bool = p != None and v == p! // <- here
print(hit(None, 1))
