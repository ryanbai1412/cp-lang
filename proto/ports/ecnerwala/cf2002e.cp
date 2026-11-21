// Codeforces 2002 E: Cosmic Rays.
// Every second each run loses its first element, so a run of length a dies at
// time a, unless the runs between it and an earlier run of the same value all
// die first, at time t: then the two merge and die at a + a' - t. A stack
// keeps runs whose death times strictly fall; a new run pops every run it
// outlives, remembering when the last one died, and merges with a same-value
// run it reaches. The strength of the prefix is the bottom run's death.
@!multitest
let n: 1..=300_000 = read()
var stack: List<(dies: Int, val: Int)> = []
var strength: List<Int> = []
for _ in 0..n:
    var dies: Int = read::<1..=1_000_000_000>()
    let val: 0..=n = read()
    var gone = 0  // when the runs popped so far died
    while let top = stack.last:
        if top.val != val and dies < top.dies: break
        if top.val == val: dies += top.dies - gone
        gone = top.dies
        stack.pop()
    stack.push((dies: dies, val: val))
    strength.push(stack[0].dies)
print(strength)
