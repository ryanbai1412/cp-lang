// CCO 2004 P2, tuned: port.cp with each score's digits parsed in one pass instead of split into
// two strings, and each game printed score by score instead of as a list of formatted strings.
type Score = (a: Int, b: Int)  // a <= b

fn score(word: Str) -> Score:
    var (p, q, second) = (0, 0, false)
    for c in word:
        if c == '-': second = true
        elif second: q = 10 * q + c.digit
        else: p = 10 * p + c.digit
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
    for (i, t) in game.rev.items:
        print("{t.a}-{t.b}", end: if i + 1 < game.len: " " else: "\n")
