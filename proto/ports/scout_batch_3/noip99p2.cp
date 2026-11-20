// NOIP 1999 Palindromic numbers: in base b, add a number to its mirror image
// until it reads the same both ways. Print how many additions that took, or
// give up after 30.
let base: 2..=16 = read()
let word: Str = read()
let digits = [c.digit(base) for c in word]                          // WISH G3: the value of a digit character

// n + mirror(n). The column sums read the same from both ends, so it does
// not matter which end of `a` is the low digit, and nothing is turned round.
fn add_mirror(a: List<Int>) -> List<Int>:
    var out: List<Int> = []
    var carry = 0
    for (x, y) in zip(a, a.rev):                                    // WISH G2: a.rev as a value
        carry += x + y
        out.push(carry % base)
        carry /= base
    if carry > 0: out.push(carry)
    out

fn steps() -> ?Int:
    var d = digits
    for k in 0..=30:
        if d == d.rev: return k                                     // WISH G2
        d = add_mirror(d)
    none

if let k = steps(): print("STEP={k}")                               // WISH G4: text and a value with no space
else: print("Impossible!")
