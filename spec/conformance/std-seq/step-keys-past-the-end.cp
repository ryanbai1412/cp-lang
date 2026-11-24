// expect: ok, output "1 0 3 0\n70"
// rule: 04b.1.10 09.3.10
// A step view's keys stop at the last visited key; past the end and negative keys are not in them (adversarial round 47).
let a = [10, 20, 30, 40, 50, 60, 70]
let s = a.step(3)
print(Int(6 in s.keys), Int(7 in s.keys), s.keys.len, Int(-3 in s.keys))
print(s.at(6))
