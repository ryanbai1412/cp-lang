// ccc20s4 Swapping Seats: fewest swaps to seat A, B and C people in three blocks around a round
// table. Try every start for the A block and both orders of B and C; for each, c[x, y] counts
// people of letter x sitting in letter y's block. A two-cycle costs one swap and each leftover
// three-cycle two.
enum Who = A | B | C
let s = read<Str of "ABC">()
domain Seat = [s.len]
let who: [Seat]Who = [Seat] i => match s[Int(i)]:
    'A' => A
    'B' => B
    _ => C
let pre = [Who] x => ([Seat] i => Int(who[i] == x)).scan(+)
let size = [Who] x => pre[x].all

fn swaps(start: Seat, order: List<Who>) -> Int:
    var block = [Who] _ => who.ring.span(start, 0)
    var at = start
    for y in order:
        block[y] = who.ring.span(at, size[y])
        at = who.ring.step(at, size[y])
    let c = [Who, Who] (x, y) => pre[x].cyclic_sum(block[y])
    let two = [(A, B), (A, C), (B, C)].sum(of: p => min(c[p.0, p.1], c[p.1, p.0]))
    let left = [(A, B), (A, C), (B, C)].sum(of: p => abs(c[p.0, p.1] - c[p.1, p.0]))
    two + 2 * (left / 3)

print([swaps(start, order) for start in Seat for order in [[A, B, C], [A, C, B]]].min()!)
