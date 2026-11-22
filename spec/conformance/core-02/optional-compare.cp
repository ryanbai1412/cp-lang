// expect: E0308 at 2
// rule: 02.4.5
let d: ?Int = None
if d + 1 < 5: print("x")
