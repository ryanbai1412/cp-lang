// EGOI '23 P8 (Guessing Game): two-phase communication. Phase 1 labels n - 1
// of n houses online; phase 2 sees the labels with two erased and names two
// candidates for the unlabelled house. The (n - 13) houses labelled 1 carry
// their index sum mod n, split into a composition of 11 over labels 2..=10
// plus one parity bit.
// original: corpus/submissions/5659039.cpp (91/100)
const PARTS = 9
const TOTAL = 11

// every way to write TOTAL as PARTS ordered non-negative digits, packed
// base 12, in increasing order
var codes: List<Int> = []
fn gen(slots: Int, left: Int, code: Int):
    if slots == 0:
        if left == 0: codes.push(code)
        return
    for d in 0..=left: gen(slots - 1, left - d, code * 12 + d)
gen(PARTS, TOTAL, 0)

fn digit(code: Int, i: Int) -> Int:
    return code / 12 ** (PARTS - 1 - i) % 12

let phase: 1..=2 = read()
let n: 13..=150_000 = read()
domain H = 0..n

if phase == 1:
    var seen: [H]Bool = false
    print(16)
    var one_sum = 0
    for _ in 0..n - 13:
        let h: H = read()
        seen[h] = true
        one_sum = (one_sum + h.idx) % n
        print(1)
    var rest: List<H> = []
    for h in H:
        if not seen[h]: rest.push(h)
    let code = codes[one_sum / 2]
    for i in 0..PARTS:
        for _ in 0..digit(code, i):
            let h: H = read()
            seen[h] = true
            print(2 + i)
    // the last label says which pair of the 12 remaining unlabelled-by-1
    // houses contains the one that never shows up
    let flag: H = read()
    seen[flag] = true
    let hidden = first(h for h in H if not seen[h])!
    let slot = count(h for h in rest if h != flag and h < hidden)
    print(11 + slot / 2)
else:
    let label: [H]0..=16 = read()
    var ones: List<H> = []
    var coded: List<H> = []
    var flags: List<H> = []
    for h in H:
        if label[h] == 1: ones.push(h)
        elif label[h] >= 11: flags.push(h)
        elif label[h] >= 2: coded.push(h)
    if ones.len != n - 13:
        // a 1-house was erased: recover the index sum and solve for it
        var code = 0
        for h in coded: code += 12 ** (10 - label[h])
        let at = codes.find(code) ?? codes.len
        let known = sum(h.idx for h in ones)
        print((known - 2 * at) % n, (known - 2 * at - 1) % n)
    elif flags.len == 2: print(flags[0], flags[1])
    else:
        let pair = (label[flags[0]] - 11) * 2
        print(coded[pair], coded[pair + 1])
