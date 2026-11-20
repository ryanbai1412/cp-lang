// cp: 0.1 1f59ac0+dirty
// Yet Another Contest 7 P5 (interactive): each of n eggs breaks from its own
// floor of 1..=n upward, all different; "? egg floor" tells whether that egg
// breaks there. Split the floors in half, sort the eggs by one question each
// (the last egg of a group needs none), and recurse, starting with the half
// nearer to the floor we are standing on.
let n: 1..=1e4 = read()
domain Egg = 1..=n
var floor_of: [Egg]Int = 0
var standing = 0                        // the floor of the latest question

fn breaks(egg: Egg, floor: Int) -> Bool:
    standing = floor
    let broke: 0..=1 = ask("?", egg, floor)
    broke == 1

// `eggs` are exactly the eggs whose floors are lo..hi
fn place(lo: Int, hi: Int, eggs: List<Egg>):
    if hi - lo == 1:
        floor_of[eggs[0]] = lo
        return
    var mid = (lo + hi) / 2
    if hi - lo <= 3: mid = if standing <= lo: lo + 1 else: hi - 1
    let upper_first = standing < mid - 1
    var lower: List<Egg> = []
    var upper: List<Egg> = []
    for i in 0..eggs.len - 1:
        if breaks(eggs[i], mid - 1): lower.push(eggs[i]) else: upper.push(eggs[i])
    let spare = eggs[eggs.len - 1]
    if upper.len == hi - mid: lower.push(spare) else: upper.push(spare)
    if upper_first:
        place(mid, hi, upper)
        place(lo, mid, lower)
    else:
        place(lo, mid, lower)
        place(mid, hi, upper)

place(1, n + 1, [e for e in Egg])
print("!", floor_of)
