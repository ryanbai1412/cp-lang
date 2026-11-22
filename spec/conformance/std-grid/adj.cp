// expect: ok, output "YES"
domain R = 0..3
domain C = 0..3
let a = [q for q in (R(1), C(1)).adj4]
let b = [q for q in (R(0), C(0)).adj4]
let c = [q for q in (R(1), C(1)).adj8]
let d = [q for q in (R(0), C(0)).adj8]
print(a == [(R(0), C(1)), (R(1), C(2)), (R(2), C(1)), (R(1), C(0))] and b == [(R(0), C(1)), (R(1), C(0))] and c.len == 8 and d == [(R(0), C(1)), (R(1), C(1)), (R(1), C(0))])
