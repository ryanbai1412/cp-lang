// expect: ok, output "102 4"
// rule: 04.2.10
// Over three axes, one table name is the 3-tuple cell and `t.keys` walks every cell.
domain A = 0..2
domain B = 0..2
domain C = 0..3
let t = [A, B, C] p => Int(p.0) * 100 + Int(p.1) * 10 + Int(p.2)
print(t[A(1), B(0), C(2)], t.keys.count(p => Int(p.2) == 2))
