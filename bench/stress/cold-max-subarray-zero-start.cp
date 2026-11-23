// Largest sum of a non-empty contiguous subarray, written cold for the Milestone 1 loop run, with
// a planted bug: the best sum starts at 0, a sentinel no all-negative array reaches.
// class: sentinel

let n: 1..=8 = read()
let a: [0..n]-9..=9 = read()
var best = 0  // planted
var here = 0
for i in 0..n:
    here = max(a[i], here + a[i])
    best max= here
print(best)

@alt(program)
fn every_range():
    var answer = a[0]
    for i in 0..n:
        var s = 0
        for j in i..n:
            s += a[j]
            answer max= s
    print(answer)
