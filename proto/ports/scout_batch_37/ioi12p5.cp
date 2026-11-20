// IOI 2012 Day 2 P2, Last Supper
// original: corpus/submissions/4951033.cpp
// FRICTION: `Colour = 0..n` is shared by five grader signatures in two separate runs,
// but n only arrives as an argument, so no file-scope `domain Colour` can be written.
@grader fn WriteAdvice(bit: Bool)
@grader fn GetRequest() -> Colour
@grader fn PutBack(c: Colour)

// Replay Belady's rule (evict the colour needed latest). A "stay" is one
// colour's unbroken spell on the scaffold: stay i < k is the initial colour
// i, stay k + t starts at request t. Its advice bit says whether the colour
// is requested again before that stay ends.
@grader
fn ComputeAdvice(req: [0..n]Colour, n: Int, k: Int, m: Int):
    var next_use: [0..n]Int = n
    var upcoming: [Colour]Int = n
    for t in (0..n).rev:
        next_use[t] = upcoming[req[t]]
        upcoming[req[t]] = t
    var stay: [Colour]?Int = none
    var needed: [0..n + k]Bool = false
    var on = Set<(Int, Colour)>()
    for c in Colour if c.to_idx < k:
        stay[c] = c.to_idx
        on.add((upcoming[c], c))
    for t in 0..n:
        let c = req[t]
        if let s = stay[c]:
            needed[s] = true
            on.remove((t, c))
        else:
            let (_, out) = on.pop_last()!
            stay[out] = none
        stay[c] = k + t
        on.add((next_use[t], c))
    for bit in needed: WriteAdvice(bit)

// Sees only the bits and one request at a time. Colours whose current stay
// is not needed again are safe to put back.
@grader
fn Assist(needed: [0..r]Bool, n: Int, k: Int, r: Int):
    var here: [Colour]Bool = false
    var spare: List<Colour> = []
    for c in Colour if c.to_idx < k:
        here[c] = true
        if not needed[c.to_idx]: spare.push(c)
    for t in 0..n:
        let c = GetRequest()
        if not here[c]:
            let out = spare.pop()!
            PutBack(out)
            here[out] = false
            here[c] = true
        if not needed[k + t]: spare.push(c)
