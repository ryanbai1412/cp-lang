// expect: E0308 at 3
let d: ?Int = none
if d + 1 < 5: print("x")
