// `with lint(...)` speaks to `cpl lint` alone: the fn runs and builds as without it.
fn pairs(xs: List<Int>) -> Int:
    with lint(unproven_list_index: allow)
    var same = 0
    for i in 0..xs.len:
        for j in 0..i:
            if xs[i] == xs[j]: same += 1
    same
print(pairs([1, 2, 1, 1]))
