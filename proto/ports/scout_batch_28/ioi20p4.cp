// IOI 2020 Packing Biscuits: k kinds of biscuit, kind i worth 2^i with
// a[i] of them in stock. Pack x bags of equal total worth y, not
// necessarily using every biscuit. For each of q scenarios print how many
// different y are possible. (Input as the IOI sample grader gives it.)
// y works exactly when, for every bit i set in y, the low i + 1 bits of y
// are at most (worth of kinds 0..=i) / x. So the y below a limit split on
// their top bit, and only a few limits ever come up at each bit.
// original: corpus/submissions/4733047.cpp
use std.seq.{scan}

const TOP = 60                              // all the biscuits are worth < 2^60
let two = [2 ** i for i in 0..=TOP]
@multitest
fn solve():
    let k: 1..=TOP = read()
    let x: 1..=1e18 = read()
    // every a[i] is at most 10^18, but so is the whole stock's worth: declaring
    // the per-kind bound would make a[i] * 2^i look as if it could overflow
    let a: [k]Int = read()
    let worth = scan([a[i] * two[i] for i in 0..k], +) where worth[k - 1] <= 1e18

    // how many possible y are less than lim and use only bits below `bits`
    @memo(hash)
    fn below(bits: 0..=TOP, lim: Int) -> Int:
        if lim <= 0: return 0
        if bits == 0: return 1
        let top = bits - 1                  // split on whether y has this bit
        let half = two[top]
        let stock = worth[min(top, k - 1)]
        let room = min(lim, 2 * half, stock / x + 1) - half
        below(top, min(lim, half)) + below(top, room)

    print(below(TOP, two[TOP]))
