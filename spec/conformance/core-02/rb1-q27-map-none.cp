// expect: ok, output "0"
// rule: 02.4.1 02.4.2
let x: ?Int = None
print(x.map(v => v * 2) ?? 0)
