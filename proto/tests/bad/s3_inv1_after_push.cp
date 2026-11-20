// Anti-spookiness Invariant 1: `push` does NOT implicitly narrow `q.last` without a syntactic guard.
var q: List<Int> = []
q.push(1)
let y: Int = q.last
print(y)
