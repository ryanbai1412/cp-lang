// expect: ok, output "3\n5 3"
// rule: R02.9.3
// Two fns' fresh results bound at the top level each own a domain: checking the second fn
// there does not hand its domain the first binding's.
struct T<P>:
    at: [P]Int
fn make(k: Int) -> T<_>:
    domain O = 0..k
    T<O>(at: [O] _ => 1)
struct U<P>:
    at: [P]Int
fn other(k: Int) -> U<_>:
    domain O = 0..k
    U<O>(at: [O] _ => 2)
let a = make(3)
print(a.at.len)
let c = other(5)
print(c.at.len, a.at.len)
