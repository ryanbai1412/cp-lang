// expect: ok, output "3\n6\n6"
// rule: 04.4.8 04.4.9 04.8.6
domain V = 1..=4
let i = V(3).r
var below = 0
for j in ..i: below += Int(j)
var upto = 0
for j in ..=i: upto += Int(j)
let a = [V] v => Int(v)
print(below)
print(upto)
print(a[..i].sum())
