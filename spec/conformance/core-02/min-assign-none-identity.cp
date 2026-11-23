// expect: ok, output "3"
// rule: 02.4.6
// `min=` on `?Int` treats `None` as the identity on both sides; `None + 1` stays `None` (rebuild 7, H-MIN-ASSIGN-OPT).
var best: ?Int = None
let none: ?Int = None
best min= 5
best min= 3
best min= none + 1
print(best!)
