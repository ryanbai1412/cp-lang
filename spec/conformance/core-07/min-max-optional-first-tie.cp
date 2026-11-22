// expect: ok, output "b -1 1"
// rule: 07.9.3
let a = [(1, 'a'), (0, 'b'), (0, 'c')]
print(a.min(by: p => p.0)!.1, List<Int>().max() ?? -1, a.keys.min(by: i => a[i].0)!)
