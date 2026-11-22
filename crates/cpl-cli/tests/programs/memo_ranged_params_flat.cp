// A memo over ranges whose product the declarations bound is a flat table
// alone; a call outside the range is still answered, uncached.
let singles: 0..=20*400 = read()
let shines: 0..=399 = read()

@memo
fn win(k: 0..=singles, s: 0..=shines) -> Bool:
    if k == 0: return s % 2 == 1
    if s == 0: return not win(k - 1, s)
    not win(k - 1, s) or not win(k, s - 1)
print(yes(win(singles, shines)))

@memo
fn ways(k: 0..=shines) -> Int:
    if k <= 1: return 1
    ways(k - 1) + ways(k - 2)
print(ways(40), ways(shines + 12))
