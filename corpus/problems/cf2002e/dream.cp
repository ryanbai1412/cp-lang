// Codeforces 2002 E: Cosmic Rays.
// Every second each run loses its first element, so a run of length a dies at
// time a, unless the runs between it and an earlier run of the same value all
// die first, at time t: then the two merge and die at a + a' - t. A stack
// keeps runs whose death times strictly fall; a new run pops every run it
// outlives, remembering when the last one died, and merges with a same-value
// run it reaches. The strength of the prefix is the bottom run's death.
@!multitest
let n: 1..=3e5 = read()
var stack: List<(dies: Int, val: 0..=n)> = []
var strength: List<Int> = []
for (died, val) in read<[n](1..=1e9, 0..=n)>():
    var dies = died
    var gone = 0  // when the runs popped so far died
    while let top = stack.last, top.val == val or dies >= top.dies:
        if top.val == val: dies += top.dies - gone
        gone = stack.pop()!.dies
    stack.push((dies, val))
    strength.push(stack.first!.dies)
print(strength)
