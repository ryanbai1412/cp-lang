// NOIP 2008 Passing Notes: a class seated in n rows of m, each pupil with a
// kindness score (the two corner pupils score 0). A note goes from the top
// left to the bottom right, moving only down or right, and a reply comes
// back up and left; no pupil in between may help twice. Print the largest
// total kindness of the helpers.
// The reply reversed is a second note, so two notes leave together and
// take one step each at a time. After d steps a note in row r is in column
// d - r, so the state is d and the two rows, the upper note first.
// original: corpus/submissions/5185022.cpp
let n: 1..=50 = read()
let m: 1..=50 = read()
let kind: [n, m]0..=100 = read()
let last = n + m - 2                        // steps from corner to corner

// the notes may share a seat only at the two corners
fn apart(d: Int, a: Int, b: Int) -> Bool:
    a < b or (a == b and (d == 0 or d == last))

// most kindness still to collect with the notes in rows a and b after d steps
@memo
fn best(d: 0..=last, a: 0..n, b: 0..n) -> ?Int:
    if d == last: return 0
    var most: ?Int = none
    for a2 in a..=a + 1:
        for b2 in b..=b + 1:
            if b2 < n and d + 1 - a2 < m and apart(d + 1, a2, b2):
                if let rest = best(d + 1, a2, b2):
                    most max= rest + kind[a2, d + 1 - a2] + kind[b2, d + 1 - b2]
    most

print(best(0, 0, 0) ?? 0)
