// expect: R0401 at 4 (checked)
// rule: 09.2.3
// The wrong-way check runs on a domain-keyed array too: `t < 5` is true then false over [V], so the search traps instead of returning a wrong cut.
domain V = 1..=4
let a: [V]Int = [V] v => Int(v) * 2
print(a.partition_point(t => t < 5))
