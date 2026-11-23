// expect: ok, output "2 2"
// rule: 02.8.3
// A table may ignore its key with `_`: `[C] _ => None` fills every cell, here with an absent optional (rebuild 7, H-ARRAY-FILL).
domain C = 1..=3
var best: [C]?Int = [C] _ => None
best[C(2)] = 2
print(best.count(x => x == None), best[C(2)]!)
