// expect: ok, output "YES YES YES\nYES YES YES\nYES"
use std.mod.{PrimePowerComb, ModComb}
let a = PrimePowerComb.new(2, 3)
let b = PrimePowerComb.new(3, 2)
print(yes(a.choose(6, 3) == 4), yes(b.choose(9, 3) == 3), yes(b.choose(5, 7) == 0))
print(yes(ModComb.new(12).choose(6, 3) == 8), yes(ModComb.new(10).choose(5, 2) == 0), yes(ModComb.new(1).choose(5, 2) == 0))
var ok = true
for m in [2, 8, 9, 12, 27, 30, 64, 100, 360]:
    let mc = ModComb.new(m)
    var row = [1]
    for n in 0..40:
        for k in 0..=n:
            if mc.choose(n, k) != row[k] % m: ok = false
        var nxt = [1]
        for k in 1..row.len: nxt.push(row[k - 1] + row[k])
        nxt.push(1)
        row = nxt
print(yes(ok))
