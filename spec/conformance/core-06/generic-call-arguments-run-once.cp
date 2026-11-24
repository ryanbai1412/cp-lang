// expect: ok, output "4 4\n4 4\n1 2"
// rule: 08.2.5
// A generic static fn learns its type argument from its argument, which still runs once: a comprehension, a std generic, and a call nested in another call's argument.
var calls = 0
fn f(x: Int) -> Int:
    calls += 1
    x
struct Box<K>(pub n: Int):
    pub fn of(xs: List<K>) -> Box<K> = Box<K>(xs.len)
let a = [1, 2, 3, 4]
let b = Box.of([f(x) for x in a])
print(b.n, calls)
calls = 0
let s = SortedBag.of([f(x) * 2 for x in a.rev])
print(s.len, calls)
calls = 0
print(Box.of([Box.of([f(1)]).n + f(2)]).n, calls)
