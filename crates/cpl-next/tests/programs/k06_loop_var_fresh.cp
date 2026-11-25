// expect: E0603
// (spec) R8 block rule: `c` is declared in the loop body, `fs` outside it, so `fs` outlives `c`. Reviewer reads R05.10.2 as legal; today's two engines print different answers: QUESTION
// A `var` declared in a loop body is a new variable each time round; closures kept in a list
// declared outside the loop must each see their own.
fn go() -> Int:
    var fs: List<() -> Int> = []
    for i in 0..3:
        var c = i * 10
        fs.push(() => c)  // <- here
        c += 1
    var s = 0
    for i in 0..3: s = s * 100 + fs[i]()
    s
print(go())
