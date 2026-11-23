// expect: ok, output "2 1 0\n30 20 10\n2 1 0"
// rule: 07.4.7
// A `.rev` passed to a generic fn still walks backwards: its keys, elements and `items` come in reversed order inside the callee, exactly as at the call site (silent wrong answer: the callee walked forwards).
fn kk<S: Sized>(s: S) -> List<Int> = List(s.keys)
fn xs<S: Seq>(s: S) -> List<Int> = List(s)
fn ks<S: Keyed>(s: S) -> List<Int> = [k for (k, x) in s.items]
let a = [10, 20, 30]
print(kk(a.rev))
print(xs(a.rev))
print(ks(a.rev))
