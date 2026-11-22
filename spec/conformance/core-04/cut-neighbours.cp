// expect: ok, output "20\n12\n4"
// rule: 04.3.2 04.8.1
domain V = 1..=4
let c = V.cuts(1)
print(Int(c.next!) * 10 + Int(c.prev!))
print(Int(c.l!) * 10 + Int(c.r!))
print(Int(V.cuts(0).l ?? V(4)))
