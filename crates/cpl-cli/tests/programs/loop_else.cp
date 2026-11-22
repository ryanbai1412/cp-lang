fn find_first(xs: List<Int>, target: Int) -> Int:
    var ans = 0
    for x in xs:
        if x == target:
            ans = x
            break
    else:
        ans = -1
    ans

fn while_search(n: Int, stop: Int) -> Int:
    var i = 0
    var ans = 0
    while i < n:
        if i == stop:
            ans = i
            break
        i += 1
    else:
        ans = -2
    ans

fn while_let_search(xs: List<Int>, stop: Int) -> Int:
    var s = [x for x in xs]
    var ans = 0
    while let x = s.pop():
        if x == stop:
            ans = x
            break
    else:
        ans = -3
    ans

fn labelled_break_skips_both_else(hit: Int) -> Int:
    var log = 0
    'outer: for i in 0..3:
        for j in 0..3:
            if i * 10 + j == hit:
                log += 1000
                break 'outer
        else:
            log += 1
    else:
        log += 100
    log

// Empty range runs else
var empty_ran = 0
for x in 0..0:
    empty_ran = 1
else:
    empty_ran = 7

// One-line loop body with else
var oneline = 0
for x in 1..=3: oneline += x
else: oneline += 100

// V.pairs (nested_for) with else
domain V = 0..3
var pairs_else = 0
for (i, j) in V.pairs:
    if Int(i) + Int(j) == 99: break
else:
    pairs_else = 42

print(
    find_first([2, 4, 6], 4),
    find_first([2, 4, 6], 5),
    while_search(5, 3),
    while_search(5, 9),
    while_let_search([1, 2, 3], 2),
    while_let_search([1, 2, 3], 9),
    labelled_break_skips_both_else(11),
    labelled_break_skips_both_else(99),
    empty_ran,
    oneline,
    pairs_else,
)
