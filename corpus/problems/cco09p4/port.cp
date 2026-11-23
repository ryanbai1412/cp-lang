// CCO 2009 Bottle Caps (interactive), as it has to be written today: the
// three-way split is three lists filled by hand. Builds; 60 conversations
// with cco09p4_judge.py accepted for this file and for 5034519.cpp.
let n: 1..=1000 = read()
domain Cap = 1..=n
domain Bottle = 1..=n
fn fit(c: Cap, b: Bottle) -> Int:
    let r: -1..=1 = ask(0, c, b)
    r
fn pair_up(caps: List<Cap>, bottles: List<Bottle>):
    if caps.len == 0: return
    let pivot = bottles[rand(0, bottles.len - 1)]
    var small: List<Cap> = []
    var big: List<Cap> = []
    var fits: ?Cap = None
    for c in caps:
        let r = fit(c, pivot)
        if r < 0: small.push(c)
        elif r > 0: big.push(c)
        else: fits = c
    let cap = fits!
    print(1, cap, pivot)
    let below = [b for b in bottles if b != pivot and fit(cap, b) > 0]
    let above = [b for b in bottles if b != pivot and fit(cap, b) < 0]
    pair_up(small, below)
    pair_up(big, above)
pair_up(Cap.all, Bottle.all)
