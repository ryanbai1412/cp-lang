// Source: CCO 2022 P3 Double Attendance, https://dmoj.ca/problem/cco22p3
// Problem: two rooms show slides during open intervals (a, b): n1, n2 <= 3e5, no overlap within a
//   room, times <= 1e9. You start in room 1 at time 0; moving between rooms takes k <= 1e9. A
//   slide counts if you spend positive time in its room while it shows; print the most slides.
// Solution: a state is (slides seen, room, whether the slide showing in the other room is
//   already counted), and only its earliest time matters. Every move sees one more slide: wait
//   for the next slide in this room, or move and catch the first uncounted slide still showing
//   at d + k (binary search on sorted starts). So a DP over the count of slides seen, layer by
//   layer, keeps each state's earliest time: 4 (n1 + n2) states, O((n1 + n2) log) in all.
// original: corpus/submissions/4801968.cpp
let (n1, n2, k): (1..=3e5, 1..=3e5, 1..=1e9) = read()
domain Room = 0..2
let first = read<[n1](a: 0..=1e9, b: 0..=1e9)>().sorted()
let second = read<[n2](a: 0..=1e9, b: 0..=1e9)>().sorted()
let slides: [Room]List<(a: Int, b: Int)> = [first, second]
domain Seen = 0..=n1 + n2

fn started(r: Room, t: Int) -> ?Int = slides[r].last_le(t, by: s => s.a)

var earliest: [Seen, Room, Bool]?Int = None
var most = 0
fn reach(u: Seen, r: Room, kept: Bool, t: Int):
    most max= u.ord
    earliest[u, r, kept] min= Some(t)

reach(Seen(1), Room(0), false, first[0].a)
for s in second:
    if k < s.b: reach(Seen(1), Room(1), false, max(s.a, k))

for u in Seen:
    let Some(v) = u.next else: break
    for (r, kept) in (Room, Bool):
        let Some(d) = earliest[u, r, kept] else: continue
        let o = Room(1 - r.ord)
        let here = started(r, d)!
        let there = started(o, d)
        if here + 1 < slides[r].len:
            let t = slides[r][here + 1].a
            reach(v, r, kept and there.is_some_and(j => t < slides[o][j].b), t)
        var c = started(o, d + k) ?? 0
        if kept and there == Some(c): c += 1
        if c < slides[o].len and d + k >= slides[o][c].b: c += 1
        if c < slides[o].len:
            let t = max(d + k, slides[o][c].a)
            reach(v, o, t < slides[r][here].b, t)
print(most)
