// expect: ok, output "-1"
// rule: 02.4.7
let d: ?Int = None
let r = (d? + 1) ?? -1
print(r)
