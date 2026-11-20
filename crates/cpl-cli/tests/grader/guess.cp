// A grader-style problem: the judge holds a hidden number in 1..=n and
// answers `compare`; `find` is what the judge calls.
@grader fn compare(x: Int) -> Int
@grader fn note(tries: List<Int>, label: Str)

var calls = 0

@grader
fn find(n: Int) -> Int:
    var lo = 1
    var hi = n
    var tries: List<Int> = []
    while lo < hi:
        let mid = (lo + hi) / 2
        calls += 1
        tries.push(mid)
        if compare(mid) < 0: lo = mid + 1 else: hi = mid
    note(tries, "done")
    lo

@grader
fn stats(scale: Int) -> (Int, List<Int>):
    (calls * scale, [calls, scale])
