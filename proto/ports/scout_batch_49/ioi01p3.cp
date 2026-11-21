// IOI 2001 Day 1 P3, Twofive
// original: corpus/submissions/4806167.cpp
type Letter = 0..25
type Cell = 0..25                    // row * 5 + column
type Shape = (0..=5, 0..=5, 0..=5, 0..=5, 0..=5)

// A grid is valid when letters A..Y, placed in order, always extend a
// staircase: row r may grow only while it stays shorter than row r - 1.
// `pin` forces some letters onto fixed cells; count the valid completions.
fn completions(pin: [Letter]?Cell) -> Int:
    // FRICTION: the natural key is `[0..5]0..=5`, but an array is not an Index,
    // so the staircase is a 5-tuple that has to be rebuilt as a list to loop over.
    @memo
    fn ways(shape: Shape) -> Int:
        let len = [shape.0, shape.1, shape.2, shape.3, shape.4]
        let letter = len.sum()
        if letter == 25: return 1
        var total = 0
        for r in 0..5 if len[r] < 5 and (r == 0 or len[r - 1] > len[r]):
            if (pin[letter] ?? r * 5 + len[r]) == r * 5 + len[r]:
                var grown = len.copy()
                grown[r] += 1
                total += ways((grown[0], grown[1], grown[2], grown[3], grown[4]))
        total
    ways((0, 0, 0, 0, 0))

var pin: [Letter]?Cell = none
var used: [Letter]Bool = false
let kind: Char = read()
if kind == 'W':
    let word: [Cell]Char = read()
    var rank = 1
    for cell in Cell:
        let letter = word[cell] - 'A'
        for smaller in 0..letter if not used[smaller]:
            pin[smaller] = cell
            rank += completions(pin)
            pin[smaller] = none
        pin[letter] = cell
        used[letter] = true
    print(rank)
else:
    var skip = read::<Int>() - 1
    var word = ""
    for cell in Cell:
        for letter in Letter if not used[letter]:
            pin[letter] = cell
            let here = completions(pin)
            if skip < here:
                used[letter] = true
                word.push('A' + letter)
                break
            skip -= here
            pin[letter] = none
    print(word)
