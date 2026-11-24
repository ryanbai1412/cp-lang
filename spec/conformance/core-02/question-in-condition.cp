// expect: E0803
// rule: 02.4.7
let d: ?Int = None
if d? > 0: print(1) // <- here
