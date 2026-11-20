// CCC 2021 S3 Lunch Concert: friend i stands at p, walks at w seconds per metre
// and hears the concert from within d metres. Choose where to play so that the
// total walking time is least.
// original: corpus/submissions/4323533.cpp
let n: 1..=2e5 = read()
let friends: [n](p: 0..=1e9, w: 1..=1000, d: 0..=1e9) = read()

fn walk(c: Int) -> Int = sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends)

// The total is convex in c. Its slope is -w for a friend who still has to walk
// right, and it gains w at each end of a friend's hearing range, so the best
// spot is the first range end where the slope stops being negative.
let ends = sorted((at, f.w) for f in friends for at in [f.p - f.d, f.p + f.d])     // WISH S9
var slope = -sum(f.w for f in friends)
for (at, w) in ends:
    slope += w
    if slope >= 0:
        print(walk(at))
        break
