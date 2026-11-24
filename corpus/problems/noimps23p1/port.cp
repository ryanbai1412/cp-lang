// Source: NOI Provincial Selection 2023 P1, Train Station,
//   https://dmoj.ca/problem/noimps23p1
// limits: 1000 ms, 512 MB
// Problem: n stations on a line and m tracks [l, r] (l < r). A train leaves station x along any
//   track through x, in either direction, may change to another track at any station both
//   cover but never turns back, and stops at the end of a track. Print, ascending, every
//   station where it can stop. n, m <= 2e5.
// Solution: rightward stops are the right ends of tracks reachable from x, where a track is
//   reachable if it covers some station in [x, r] of a reachable track. Riding a track sweeps
//   its stations not yet swept, found with an IntervalSet's `next_gap`, and boards the tracks
//   that start there, so each station is swept once. Leftward stops are the same on the
//   mirrored line. O((n + m) log n).
// original: corpus/submissions/5556659.cpp
use std.interval.{IntervalSet}

let (n, m): (2..=200_000, 1..=200_000) = read()
domain Station = 1..=n
let x: Station = read()
domain Track = 0..m
let tracks: [Track](l: Station, r: Station) = read()

fn mirror(s: Station) -> Station = Station.nth(Station.count - 1 - s.ord)

/// Where a train from `x` riding only toward higher stations of `line` can stop.
fn stops(x: Station, line: [Track](l: Station, r: Station)) -> [Station]Bool:
    let starting = line.map(.l).buckets()
    var swept = IntervalSet<Station>()
    var stop: [Station]Bool = false
    var riding = [t for t in Track if line[t].l <= x < line[t].r]
    while let t = riding.pop():
        let (l, r) = line[t]
        stop[r] = true
        var from = max(x, l)
        while let s = swept.next_gap(from) and s <= r:
            swept.add(s..=s)
            riding.extend(starting[s])
            from = s
    stop[x] = false
    stop

let right = stops(x, tracks)
let left = stops(mirror(x), [Track] t => (l: mirror(tracks[t].r), r: mirror(tracks[t].l)))
print([s for s in Station if right[s] or left[mirror(s)]])
