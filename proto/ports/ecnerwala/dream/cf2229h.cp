// Codeforces 2229 H: Wowee Binary String.
// A deletion never crosses a kept character, so t is obtainable iff it embeds in a
// completion of s with every gap (before, between and after the kept characters)
// holding an even number of 1s; a gap with a '?' can always be made even. Read t
// left to right, tracking the cuts of s where the embedding may continue. Of two live
// cuts with the same parity of fixed 1s before them, the earlier does all the later
// does, so a Reach is the earliest live cut of each parity: a DFA on O(n^2) states
// whose paths spell the distinct t. O(n^2).
type Z = Mod<998_244_353>

@!multitest
let n: 1..=3000 = read()
domain Pos = 0..n
domain Cut = Pos.bounds
let s: [Pos]Char = read()
let ones = Prefix(s.map(ch => ch == '1'))
let wild = Prefix(s.map(ch => ch == '?'))
fn evenable(gap: Range<Pos>) -> Bool = wild.sum(gap) > 0 or ones.sum(gap) % 2 == 0

domain Side = 0..2
type Reach = [Side]?Cut
fn reach(cuts: List<Cut>) -> Reach = [Side] q => cuts.filter(x => ones.prefix(x) % 2 == q).min()

@memo fn step(x: Cut, c: Char) -> Reach =                 // O(n), once per (x, c)
    reach([j.after for j in x..Cut.last if s[j] in [c, '?'] and evenable(x..j.before)])
fn read_char(at: Reach, c: Char) -> Reach = reach([y for x in at.flatten() for y in step(x, c).flatten()])

@memo fn words(at: Reach) -> Z:
    let live = at.flatten()
    if live.is_empty: return 0
    let stop: Z = if live.any(x => evenable(x..Cut.last)): 1 else: 0
    stop + words(read_char(at, '0')) + words(read_char(at, '1'))

print(words(reach([Cut.first])))
