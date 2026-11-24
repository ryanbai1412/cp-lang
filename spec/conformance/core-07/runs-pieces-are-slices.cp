// expect: ok, output "2 2 2 1\n2 1 3\n2 3 1\n1\n3\n4 2 2 2 1"
// rule: 07.7.10
let xs = [1, 3, 2, 4, 5, 7, 6]
print([r.len for r in xs.runs(by: x => x % 2)])
let s = "aabccc"
print([r.len for r in s.runs(by: c => c)])
domain D = 0..6
let a: [D]Int = [1, 1, 2, 2, 2, 3]
print([r.len for r in a.runs(by: x => x)])
for r in xs.runs(by: y => y > 1):
    print(r.first!)
let held = xs.runs(by: x => x % 2)
print(held.count(), [r.len for r in held])
