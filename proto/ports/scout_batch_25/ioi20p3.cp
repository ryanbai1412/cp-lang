// IOI '20 P3 - Carnival Tickets (DMOJ: ioi20p3)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// n colours (n even), m tickets per colour, sorted. In each of k rounds hand
// over one ticket of every colour; the game master picks b to minimise
// sum |a - b|, which is (sum of the upper half) - (sum of the lower half).
// Allocate tickets to rounds to maximise the total prize.
//
// Exchange argument: begin with every colour giving its k smallest tickets
// as "minus". Turning colour c's j-th upgrade on swaps its largest unused
// minus for its largest unused ticket and gains x[c][m-1-j] + x[c][k-1-j];
// the gains of one colour are non-increasing, so the best n*k/2 upgrades over
// all colours are just the n*k/2 largest gains. Rounds are then dealt by
// always giving the pluses to the n/2 colours with the most pluses left.
@grader
fn find_maximum(k: Int, x: [Colour, Ticket]Int) -> (prize: Int, round: [Colour, Ticket]?Round):
    domain Colour, Ticket          // inferred from the shape of x
    domain Round = 0..k
    let (n, m) = (Colour.len, Ticket.len)

    let gain = (c: Colour, j: Int) => x[c, Ticket(m - 1 - j)] + x[c, Ticket(k - 1 - j)]
    let ups = sorted([(c, j) for c in Colour for j in 0..k], by: (c, j) => desc(gain(c, j)))
        .take(n * k / 2)
    var plus: [Colour]Int = counts(c for (c, _) in ups)

    let prize = sum(gain(c, j) for (c, j) in ups) - sum(x[c, Ticket(j)] for c in Colour for j in 0..k)

    var round: [Colour, Ticket]?Round = none
    var (lo, hi): ([Colour]Int, [Colour]Int) = (0, m - 1)     // next unused small / large ticket
    for r in Round:
        let by_plus = sorted(Colour, by: c => desc(plus[c]))
        for (rank, c) in by_plus.enumerate:
            if rank < n / 2:
                round[c, Ticket(hi[c])] = r
                hi[c] -= 1
                plus[c] -= 1
            else:
                round[c, Ticket(lo[c])] = r
                lo[c] += 1
    (prize, round)
