// A piece of a `.rev` is a Seq like the walk it cuts, so generic Seq fns
// take it, nested pieces too (R07.4.7).
let a = [10, 20, 30, 40, 50]
print(sum(a.rev.take(3)), sum(a.rev.take(3).take(2)), sum(a.drop(1).rev.take(2)))
print(a.rev.drop(2).max()!, List(a.rev.take(4).drop(1)))
