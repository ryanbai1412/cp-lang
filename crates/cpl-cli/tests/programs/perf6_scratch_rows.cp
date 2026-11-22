// A scratch list or row made anew on every turn of a loop keeps one buffer
// in the C++. Nothing it held may leak into the next turn, and one that
// leaves the loop is built fresh.
let n: Int = read()
var kept: List<List<Int>> = []
var total = 0
for round in 0..n:
    var xs: List<Int> = []
    var seen: [0..n + 1]Bool = false
    var cnt: [0..round + 2]Int = round
    var best: [0..3]?Int
    for i in 0..round:
        xs.push(i * round)
        seen[i] = true
    if round == 2: continue
    best[round % 3] = round
    var moved: List<Int> = []
    moved.push(round)
    kept.push(moved)
    var marks = 0
    for i in 0..=n: marks += Int(seen[i])
    total += sum(xs) + sum(cnt) + marks
    print(round, xs, cnt, marks, [if let v = x: "{v}" else: "none" for x in best])
    if round == 4: break
print(total, kept)

fn tally(k: Int) -> Int:
    var acc = 0
    for r in 0..k:
        var row: [0..k]Int = 1
        var tmp: List<Int> = []
        for i in 0..r:
            row[i] += r
            tmp.push(row[i])
        if tmp.len == 3: return acc + sum(row)
        acc += sum(row) + tmp.len
    acc
print(tally(2), tally(3), tally(6))

// A row a closure reads is not shared.
fn shifted(k: Int) -> List<Int>:
    var out: List<Int> = []
    for r in 0..k:
        var row: [0..k]Int = r
        row[r] = 9
        let at: fn(Int) -> Int = i => row[i] + 1
        out.push(at(0) + at(r))
    out
print(shifted(3))
