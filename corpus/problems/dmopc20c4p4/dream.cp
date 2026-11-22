// dmopc20c4p4: after each prefix, the fewest adjacent swaps that make it valid. A value equal to
// the current run length extends the run; a larger one waits on a stack and is pulled back
// (paying its distance) once the run must grow past what is waiting.
let n = read<1..=500_000>()
var run = 0
var cost = 0
var waiting: List<Int> = []
var out: List<Int> = []
for (i, x) in read<[n]Int>().items:
    while run < x:
        cost += i - waiting.pop()!
        run += 1
    if x == run:
        run += 1
    else:
        cost += run - x - 1
        waiting.push(i)
    out.push(cost)
print(out)
