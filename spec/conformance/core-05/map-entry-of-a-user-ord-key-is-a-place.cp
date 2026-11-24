// expect: ok, output "3 3 11 17 1 -1\n17 0\n10 1"
// An entry of a map whose key type has a user `Ord` is a place like any other: it is
// found by that `Ord`, then pushed to, popped, assigned into and lent.
struct P(x: Int)
impl Ord for P:
    fn lt(a, b) = b.x < a.x
struct Q(p: P, tag: Int)
impl Ord for Q:
    fn lt(a, b) = a.tag < b.tag
fn take(mut xs: List<Int>) -> ?Int = xs.pop()
fn run():
    var u = Map<P, List<Int>>()
    u[P(2)] = [10, 11]
    u[P(5)] = [1]
    u[P(2)].push(3)
    u[P(2)][0] += 7
    print(u[P(2)].len, u[P(2)].pop()!, u[P(2)].pop()!, u[P(2)][0], u[P(5)].pop()!, u[P(5)].pop() ?? -1)
    print(take(mut u[P(2)])!, u[P(2)].len)
    var w = Map<Q, Map<P, Int>>()
    w[Q(P(1), 4)] = Map<P, Int>()
    w[Q(P(1), 4)][P(3)] = 9
    w[Q(P(1), 4)][P(3)] += 1
    print(w[Q(P(1), 4)][P(3)], w.len)
run()
