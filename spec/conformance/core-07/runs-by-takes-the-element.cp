// expect: ok, output "1 2 1\n2 1\n2 2\n1\n2\n3 9"
// rule: 07.7.9
// `runs(by:)` called as a method keeps its closure's parameter the receiver's element, on a `List`, a `Str` and a `[D]T`; a bare `S: Indexed` bound gives the fn's `T` that element (`xs.nth(k)`).
let xs = [1, 2, 2, 5]
print([r.len for r in xs.runs(by: x => x % 2)])
let s = "aab"
print([r.len for r in s.runs(by: c => c.ord)])
let a = [0..4] i => i / 2
print([r.len for r in a.runs(by: v => v * 3)])
for r in xs.runs(by: y => y > 1):
    print(r.first!)
print(runs(xs, by: x => x % 2).count(), xs.nth(3) + 4)
