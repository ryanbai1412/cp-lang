// expect: ok, output "12 5 12\n1 1\n12"
// rule: R04.2.10
// A record of domains is the product domain named by its factors, walked row-major with the last
// factor fastest; its positions are records, and a range factor is a domain of its own.
domain C = 0..3
domain State = (comps: 0..=2, used: C)
var best: [State]Int = [State] s => 0
for s in State: best[s] = s.comps.ord * 10 + s.used.ord
let p = State.nth(5)
print(State.count + 3, p.ord, best[p])
let q: State = (comps: p.comps, used: C.first!)
print(q.comps.ord, State.nth(4).used.ord)
fn local(k: Int) -> Int:
    domain P = (a: 0..k, b: 1..=3)
    let t: [P]Int = [P] x => 1
    t.len
print(local(4))
