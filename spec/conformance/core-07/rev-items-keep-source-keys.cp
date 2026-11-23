// expect: ok, output "4 40\n3 30\n2 20\n1 10"
// rule: 07.4.7
// `.items` of `a.rev` walks backwards and keeps `a`'s own keys.
domain V = 1..=4
let a = [V] v => 10 * Int(v)
for (v, x) in a.rev.items:
    print(Int(v), x)
