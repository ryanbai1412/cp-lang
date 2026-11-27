// expect: ok, output "0 1 2\n18\n2 6 10\n3 4 5\n3 6"
// An open range starts at 0 or its domain's first value and runs to its domain's end;
// `by` steps a range in a `for` as it does a range value.
domain V = 0..6
domain W = 1..=4
var below: List<Int> = []
for k in ..3: below.push(k)
print(below)
var s = 0
for i in 0..10 by 3: s += i
print(s)
var ds: List<Int> = []
for d in 2..=10 by 4: ds.push(d)
print(ds)
var vs: List<Int> = []
for j in V(3)..: vs.push(j.ord)
print(vs)
let c = W(3).r
var lt = 0
for j in ..c: lt += Int(j)
var le = 0
for j in ..=c: le += Int(j)
print(lt, le)
