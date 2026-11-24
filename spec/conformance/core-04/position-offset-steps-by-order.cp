// expect: ok, output "3 1 none 4 none\n2\n3\n4"
// rule: 04.3.1
// `offset`, `ahead` and `back` step by order, not by label, and give `None` off either end.
domain V = 1..=4
let v = V(2)
print(v.offset(1), v.offset(-1), v.back(2), v.ahead(2), v.ahead(3))
for k in 0..3:
    if let Some(u) = V(2).offset(k): print(u)
