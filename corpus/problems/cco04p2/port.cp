// Source: CCO 2004 P2, Hockey Scores, https://dmoj.ca/problem/cco04p2
// limits: 1000 ms, 32 MB
// Problem: multitest; each test has s <= 1000 scores `x-y`, each possibly written in either
//   order, taken from games in progress or finished. Print the fewest games such that every
//   score occurs in one, then each game's scores on a line (a valid scenario, any one).
// Solution: write each score as (min, max); scores fit one game iff they form a chain with both
//   numbers non-decreasing. Take scores largest first and put each on the row whose front has
//   the smallest second number that is still at least its own, by binary search over the
//   fronts, or open a new row; rows are printed reversed. O(s log s).
// original: corpus/submissions/5430096.cpp
// Bounds: s: 1..=1000
type Score = (a: Int, b: Int)  // a <= b

fn score(word: Str) -> Score:
    let (x, y) = word.split_once('-')!
    let (p, q) = (x.parse_int()!, y.parse_int()!)
    (a: min(p, q), b: max(p, q))

@!multitest
let s: 1..=1000 = read()
let words: [s]Str = read()
var scores = [score(w) for w in words]
scores.sort()

// Largest score first, each onto the game whose latest score has the smallest b still at least
// its own. The games stay ordered by that b, so a binary search finds it (patience sorting).
var games: List<List<Score>> = []
for t in scores.rev:
    let g = Int(games.bsearch(game => game.last!.b >= t.b))
    if g < games.len: games[g].push(t)
    else: games.push([t])
print(games.len)
for game in games:
    print(["{t.a}-{t.b}" for t in game.rev])
