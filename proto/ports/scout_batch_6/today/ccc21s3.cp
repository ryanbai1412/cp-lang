// CCC 2021 S3 Lunch Concert: friend i stands at p, walks at w seconds per metre
// and hears the concert from within d metres. Choose where to play so that the
// total walking time is least.
// original: corpus/submissions/4323533.cpp
type Friend = (p: 0..=1e9, w: 1..=1000, d: 0..=1e9)
let n: 1..=2e5 = read()
let friends: [n]Friend = read()

fn walk(c: Int) -> Int = sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends)

// The total is convex in c. Its slope is -w for a friend who still has to walk
// right, and it gains w at each end of a friend's hearing range, so the best
// spot is the first range end where the slope stops being negative.
var ends: List<(at: Int, w: Int)> = []
for f in friends:
    ends.push((f.p - f.d, f.w))
    ends.push((f.p + f.d, f.w))
var slope = -sum(f.w for f in friends)
for e in sorted(ends):
    slope += e.w
    if slope >= 0:
        print(walk(e.at))
        break
