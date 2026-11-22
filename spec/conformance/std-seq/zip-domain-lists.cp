// expect: ok, output "4"
// rule: std.seq.zip
domain V = 1..=3
let a: List<V> = [v for v in V]
let b: List<V> = [v for v in V.rev]
var ans = 0
for (x, y) in zip(a, b):
    if x == V.first!: ans = Int(x) + Int(y)
print(ans)
