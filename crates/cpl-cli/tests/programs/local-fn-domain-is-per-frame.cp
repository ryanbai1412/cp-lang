// A runtime domain declared in a recursive local fn is per frame: the nested
// calls inside the comprehension must not change the lengths max_matching reads.
use std.flow.{max_matching}
for t in 0..2:
    var calls = 0
    fn depth(n: Int) -> Bool:
        calls += 1
        if n == 0: return true
        domain Want = 0..n
        domain Have = 0..n + 1
        let fits = [Want] w => [h for h in Have if h.ord <= w.ord + 1 and depth(n - 1)]
        max_matching(fits).size == n
    print(yes(depth(3 + t)), calls)
