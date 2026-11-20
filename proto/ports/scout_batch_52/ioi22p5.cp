// IOI '22 P5 (Insects): find minimum cardinality of any insect type inside an interactive
// machine using move_inside / move_outside / press_button queries.
// Shuffle indices, count distinct types in one pass, then binary-search the minimum
// multiplicity while pruning the candidate pool on failed probes.
// original: corpus/submissions/4786085.cpp

@grader fn move_inside(i: Int)
@grader fn move_outside(i: Int)
@grader fn press_button() -> Int

@grader
fn min_cardinality(n: Int) -> Int:
    domain Bug = 0..n
    var perm = List(Bug)
    shuffle(mut perm)
    var inside = Set<Bug>()

    fn put_in(i: Bug):
        move_inside(perm[i.to_idx].to_idx)
        inside.add(i)

    fn take_out(i: Bug):
        move_outside(perm[i.to_idx].to_idx)
        inside.remove(i)

    for i in Bug:
        put_in(i)
        if press_button() > 1: take_out(i)

    let k = inside.len
    var lo = 1
    var hi = n / k
    var pool = Set<Bug>(Bug)

    while lo < hi:
        let mid = max(lo + 1, (lo + hi) / 2)
        let outside = [x for x in pool if x not in inside]
        var added: List<Bug> = []
        for x in outside:
            if inside.len == k * mid: break
            put_in(x)
            if press_button() > mid:
                take_out(x)
            else:
                added.push(x)
        if inside.len == k * mid:
            lo = mid
        else:
            hi = mid - 1
            pool = inside.copy()
            for x in added: take_out(x)
    lo
