# CCC 2020 S4 Swapping Seats: people A, B and C sit round a table. Each swap exchanges two people.
# Make every letter sit contiguously with the fewest swaps.
# Fix where the A block starts and whether B or C follows. Count c[x][y], how many people of letter
# x sit in letter y's block. A swap fixes two misplaced people when they sit in each other's blocks;
# whatever remains forms 3-cycles, which take two swaps each.
enum Who = A | B | C
let seats = read Str.map(ch => Who.parse(ch)!)
let size = seats.tally()

fn swaps(blocks: [Who]RingSlice) -> Int:
    let c = [Who, Who] (x, y) => seats.ring[blocks[y]].count(x)
    let pairs = Who.pairs().sum(of: (x, y) => min(c[x, y], c[y, x]))
    let left = Who.pairs().sum(of: (x, y) => c[x, y] - min(c[x, y], c[y, x]))
    pairs + 2 * left / 3

print(min(for start in seats.keys for order in [[A, B, C], [A, C, B]]:
    swaps(seats.ring.from(start).blocks(order.map(w => size[w])).keyed(order))))
