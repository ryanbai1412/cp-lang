// expect: ok, output "YES\nYES\nYES\nYES\n-2\n4\n10 1 5"
domain N = 0..3
domain M = 0..1
let a: [N]Int = [5, 1, 4]
var t = BeatsSegTree<N>(a)
t.set(N(1), 9)
print(yes(t.all_max == Some(9)))
t = BeatsSegTree<N>(a)
t.assign(N(0)..N(2), 0)
print(yes(t.all_sum == 4))
t = BeatsSegTree<N>(a)
t.chdiv(N(0)..=N(2), 2)
print(yes(t.all_sum == 4))
t = BeatsSegTree<N>(a)
t.chmod(N(0)..=N(2), 3)
print(yes(t.all_sum == 4))
let b: [M]Int = [-3]
var u = BeatsSegTree<M>(b)
u.chdiv(M(0)..=M(0), 2)
print(u.get(M(0)))
t = BeatsSegTree<N>(a)
t.chmod(N(0)..=N(2), 3)
print(t.all_sum)
t = BeatsSegTree<N>(a)
print(t.all_sum, t.all_min ?? -1, t.all_max ?? -1)
