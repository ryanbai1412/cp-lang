// JOI 2017 Final Semiexpress: a line of stations 1..=stations. A local train
// stops everywhere and takes `slow` per station; the express stops at the n
// given stations and takes `fast`; the semiexpress takes `semi` and stops at
// the express stops and at k - n more stations that we choose. Print how many
// stations other than the first can be reached from it within `limit`,
// changing trains freely and never going back.
// original: corpus/submissions/5331716.cpp
let (stations, n, k): (2..=1e9, 2..=3000, n..=3000) = read()                        // WISH S8
let (slow, fast, semi): (1..=1e9, 1..=1e9, 1..=1e9) = read()                        // WISH S8
let limit: 1..=1e18 = read()
let express: [n]1..=stations = read() where express.is_increasing                   // WISH S4: no way to say "sorted" to the generator

// Between two express stops, a stop serves the stations from itself up to
// where the local train runs out of time, and the next stop is best put right
// after those. Each stop serves no more than the one before it, so the best
// k - n further stops are simply the k - n that serve the most.
let spare = k - n
var reached = 0                         // served by the express stops themselves
var extra: List<Int> = []               // served by each further stop
for (from, to) in express.adjacent:                                                 // WISH S17
    var at = from                       // the next stop of this stretch
    for stop in 0..=spare:
        let left = limit - fast * (from - 1) - semi * (at - from)
        if at == to or left < 0: break
        let served = min(left / slow + 1, to - at)
        if stop == 0: reached += served
        else: extra.push(served)
        at += served
if fast * (stations - 1) <= limit: reached += 1
print(reached - 1 + sum(largest(extra, spare)))                                     // WISH S19
