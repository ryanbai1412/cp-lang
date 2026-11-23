// JOI 2017 Final Semiexpress: a line of stations 1..=stations. A local train
// stops everywhere and takes `slow` per station; the express stops at the n
// given stations and takes `fast`; the semiexpress takes `semi` and stops at
// the express stops and at k - n more stations that we choose. Print how many
// stations other than the first can be reached from it within `limit`,
// changing trains freely and never going back.
// original: corpus/submissions/5331716.cpp
let stations: 2..=1e9 = read()
let n: 2..=3000 = read()
let k: n..=3000 = read()
let slow: 1..=1e9 = read()
let fast: 1..=1e9 = read()
let semi: 1..=1e9 = read()
let limit: 1..=1e18 = read()
domain E = 1..=n

fn random_stops() -> [E]1..=stations:
    var at: [E]1..=stations = 1
    for e in E: at[e] = rand(1, stations)
    at.sort()
    at[E.first!] = 1
    at[E.last!] = stations
    at
let express: [E]1..=stations = read(gen: random_stops)  // Between two express stops, a stop serves the stations from itself up to
assert all(express[e] < express[f] for e in E if let f = e + 1)
// where the local train runs out of time, and the next stop is best put right
// after those. Each stop serves no more than the one before it, so the best
// k - n further stops are simply the k - n that serve the most.
let spare = k - n
var reached = 0            // served by the express stops themselves
var extra: List<Int> = []  // served by each further stop
for e in E:
    if let f = e + 1:
        let from = express[e]
        var at = from  // the next stop of this stretch
        for stop in 0..=spare:
            let left = limit - fast * (from - 1) - semi * (at - from)
            if at == express[f] or left < 0: break
            let served = min(left / slow + 1, express[f] - at)
            if stop == 0: reached += served
            else: extra.push(served)
            at += served
if fast * (stations - 1) <= limit: reached += 1
sort(mut extra, by: s => -s)
print(reached - 1 + sum(extra[i] for i in 0..min(spare, extra.len)))
