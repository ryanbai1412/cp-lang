// Source: JOI 2017 Final P2 (Semiexpress), https://dmoj.ca/problem/joi16fp2
// limits: 1000 ms, 512 MB
// Problem: stations 1..=N (N <= 1e9). The local takes A per station and stops everywhere, the
//   express takes B and stops at the M given stations, the semiexpress takes C (B < C < A) and
//   stops at exactly K stations including the express ones (M <= K <= 3000). Choose its other
//   stops to maximise the stations besides 1 reachable from 1 within T <= 1e18.
// Solution: in the stretch after express stop e, a stop at reaches time B(e - 1) + C(at - e)
//   and then serves min(left / A + 1, next express - at) stations by local, so the next stop
//   goes right after them. Each further stop serves no more than the one before, so make up to
//   K - M per stretch and add the K - M largest. O(MK log(MK)).
// original: corpus/submissions/5331716.cpp
// Bounds: stations: 2..=1e9; n: 2..=3000; k: n..=3000; slow, fast, semi: 1..=1e9;
//   limit: 1..=1e18
let stations: 2..=1e9 = read()
let n: 2..=3000 = read()
let k: n..=3000 = read()
let (slow, fast, semi): (1..=1e9, 1..=1e9, 1..=1e9) = read()
assert fast < semi < slow
let limit: 1..=1e18 = read()
domain E = 1..=n

fn random_stops() -> [E]1..=stations:
    var at: [E]1..=stations = 1
    for e in E: at[e] = rand(1, stations)
    at.sort()
    at[E.first!] = 1
    at[E.last!] = stations
    at
let express: [E]1..=stations = read(gen: random_stops)
assert all(express[e] < express[f] for e in E if let f = E.at(e + 1))

// In each stretch the express stop and then each further semiexpress stop serve the stations
// up to where the local runs out of time; a further stop serves no more than the one before.
let spare = k - n
var reached = 0           // stations served from the express stops themselves
var gains: List<Int> = [] // stations served by each possible further stop
for e in E:
    let Some(f) = E.at(e + 1) else: continue
    let (from, to) = (express[e], express[f])
    var at = from
    for stop in 0..=spare:
        let left = limit - fast * (from - 1) - semi * (at - from)
        if at == to or left < 0: break
        let served = min(left / slow + 1, to - at)
        if stop == 0: reached += served
        else: gains.push(served)
        at += served
let last = if fast * (stations - 1) <= limit: 1 else: 0
print(reached - 1 + last + gains.sorted().rev.take(spare).sum())
