// expect: ok, output "10 5"
// rule: 07.4.9 07.4.11
// A map view calls `f` only on what is read: `m.drop(1)` never reads element 0, so `10 / 0` never runs. A trapping `f` has an effect, so it is never read ahead (A6).
let a = [0, 1, 2]
let m = a.map(x => 10 / x)
print(m.drop(1))
