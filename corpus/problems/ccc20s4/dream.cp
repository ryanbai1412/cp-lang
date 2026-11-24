// CCC 2020 S4 Swapping Seats: people A, B and C sit round a table. Each swap exchanges two people.
// Make every letter sit contiguously with the fewest swaps.
// Fix where the A block starts and whether B or C follows. Count c[x, y], how many people of letter
// x sit in letter y's block. A swap fixes two misplaced people when they sit in each other's blocks;
// whatever remains forms 3-cycles, which take two swaps each.
let s: Str = read()
let n = s.len
domain Who = 0..3
let seat = [Who(ch.ord - 'A'.ord) for ch in s]
// before[w][i]: people of w in the first i seats of the doubled table.
var before: [Who]List<Int> = [Who] _ => [0]
for i in 0..2 * n:
    for w in Who: before[w].push(before[w].last! + Int(seat[i % n] == w))
fn among(w: Who, from: Int, len: Int) -> Int = before[w][from + len] - before[w][from]
let size = [Who] w => among(w, 0, n)

fn swaps(at: Int, order: List<Who>) -> Int:
    var start: [Who]Int = 0
    var from = at
    for w in order:
        start[w] = from
        from += size[w]
    let c = [Who, Who] (x, y) => among(x, start[y], size[y])
    let pairs = [min(c[x, y], c[y, x]) for x in Who for y in Who if x < y].sum()
    let wrong = [c[x, y] for x in Who for y in Who if x != y].sum()
    pairs + 2 * (wrong - 2 * pairs) / 3

let orders = [[Who(0), Who(1), Who(2)], [Who(0), Who(2), Who(1)]]
print([swaps(at, order) for at in 0..n for order in orders].min()!)
