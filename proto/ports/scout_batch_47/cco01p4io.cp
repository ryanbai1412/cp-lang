// CCO '01 P4 — Two-Egg Drop (Interactive) (DMOJ: cco01p4io)
// original: corpus/submissions/5034487.cpp
// CP Lang 0.2.2 (locked spec, docs/one-page-cheat-sheet.md)
//
// Interactive problem: query GetN and GetD to receive building height n <= 100
// and drop budget d, then identify the critical floor in 1..=n using at most 2
// broken balls and at most d queries ("Drop x" -> 0/1, then "Decide x").
let n: 1..=100 = ask("GetN")
let d: 1..=100 = ask("GetD")

// dp(len) = minimum worst-case drops to locate the critical floor among `len`
// remaining candidates with 2 balls, paired with the optimal first offset.
@memo
fn dp(len: 0..=100) -> (drops: Int, step: Int):
    if len <= 1: return (0, 0)
    var best_cost: ?Int = none
    var best_step = 0
    for i in 0..len:
        let cost = max(i, dp(len - i - 1).drops) + 1
        if best_cost == none or cost <= best_cost!:
            best_cost = cost
            best_step = i
    (best_cost!, best_step)

assert dp(n).drops <= d

fn search(l: Int, r: Int):
    if l == r:
        print("Decide", l)
        return
    let i = dp(r - l).step
    let broke: 0..=1 = ask("Drop", l + i)
    if broke == 1:
        for j in l..l + i:
            let b: 0..=1 = ask("Drop", j)
            if b == 1:
                print("Decide", j)
                return
        print("Decide", l + i)
    else:
        search(l + i + 1, r)

search(1, n)

@interactor
fn judge(mut sol: Interactor):
    let hidden_n: 1..=100 = read()
    let hidden_f: 1..=hidden_n = read()
    let limit = 20
    var drops = 0
    var broken = 0
    while true:
        let cmd: Str = sol.recv()
        if cmd == "GetN": sol.send(hidden_n)
        elif cmd == "GetD": sol.send(limit)
        elif cmd == "Drop":
            let x: Int = sol.recv()
            drops += 1
            assert drops <= limit and broken < 2
            let b = Int(x >= hidden_f)
            broken += b
            sol.send(b)
        elif cmd == "Decide":
            let x: Int = sol.recv()
            assert x == hidden_f
            return
