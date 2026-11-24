// expect: ok, output "1 1 1\n1 1 1\n1 1 1"
// rule: 08.2.9
// `if A: Indexed`, `Sized` and `Bidir` hold inside a generic fn for the built-in sequences it is
// called with, a domain passed as a value among them.
fn fi<A>(a: A) -> Int:
    if A: Indexed: return 1
    0
fn fs<A>(a: A) -> Int:
    if A: Sized: return 1
    0
fn fb<A>(a: A) -> Int:
    if A: Bidir: return 1
    0
domain R = 0..4
print(fi(R), fi([1]), fi(0..3))
print(fs(R), fs([1]), fs(0..3))
print(fb(R), fb([1]), fb(0..3))
