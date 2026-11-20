// CCO 2009 Bottle Caps (interactive): n caps and n bottles, each cap fits
// exactly one bottle. "0 c b" asks how cap c compares with bottle b; print
// "1 c b" for every pair that fits. Caps cannot be compared with caps, nor
// bottles with bottles.
let n: 1..=1000 = read()
domain Cap = 1..=n
domain Bottle = 1..=n

// -1: the cap is too small, 0: it fits, 1: it is too big
fn try(c: Cap, b: Bottle) -> Order: ask(0, c, b)                    // WISH G22: an Order read from -1, 0, 1

// Quicksort on both piles at once: a random bottle splits the caps, and the
// cap that fits it splits the bottles the same way.
fn pair_up(caps: List<Cap>, bottles: List<Bottle>):
    let pivot = bottles.pick() else: return                         // WISH G23: a random element
    let (small, fits, big) = split(caps, by: c => try(c, pivot))    // WISH G22: a three-way split
    let cap = fits.only                                             // WISH G24
    print(1, cap, pivot)
    let (below, _, above) = split(bottles, by: b => try(cap, b).flip)
    pair_up(small, below)
    pair_up(big, above)

pair_up(List(Cap), List(Bottle))                                    // WISH G5: a list of a whole domain
