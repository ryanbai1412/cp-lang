// Batch #13: a `match` or `if` arm that leaves, where the value is the whole statement.
enum Op: Add(Int) | Skip | Stop
fn run(ops: List<Op>, limit: Int) -> Int:
    var total = 0
    var seen: List<Int> = []
    for op in ops:
        let x: Int = match op:
            Add(k) if k > limit => return -k
            Add(k) => k
            Skip =>
                seen.push(total)
                continue
            Stop => break
        total += x
        let half = if x % 2 == 0: x / 2 else: continue
        seen.push(half)
    total * 100 + seen.len
print(run([Add(2), Skip, Add(5), Add(4), Stop, Add(9)], 10))
print(run([Add(2), Add(50)], 10))
var best = 0
'rows: for r in 0..3:
    for c in 0..3:
        best = match r * c:
            4 => break 'rows
            v if v % 2 == 1 => continue
            v => max(best, v + c)
print(best)
