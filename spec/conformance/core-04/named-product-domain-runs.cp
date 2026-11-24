// expect: ok, output "6"
// rule: 04.2.10
// A named product domain is an index space the run and build both handle.
domain State = (comps: 0..=2, used: 0..2)
let best: [State]Int = [State] s => 0
print(best.len)
