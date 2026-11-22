// expect: ok, output "31\n4\n1"
// rule: 04.3.1 04.8.1
domain V = 1..=4
let v = V(2)
print(Int(v.next!) * 10 + Int(v.prev!))
print(Int(v.next(2)!))
print(Int(V(4).next ?? V(1)))
