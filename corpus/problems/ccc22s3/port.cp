// Source: CCC 2022 S3 (Good Samples), https://dmoj.ca/problem/ccc22s3
// Problem: print n <= 1e6 pitches in 1..=m (m <= n) with exactly k <= 1e18 good samples
//   (contiguous runs with no repeated pitch), or -1 if there is no such piece.
// Solution: the good samples ending at note i are 1 plus the distinct notes just before it, so
//   1 to min(i + 1, m), and repeating the note `take` back gives exactly take. Each note in
//   turn takes the most it can (a fresh pitch of the cycle when it takes all) while leaving 1
//   for every later note; print -1 unless that uses up exactly k. O(n).
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
let k: 1..=1e18 = read()
domain Note = 0..n
domain Pitch = 1..=m

// The good runs ending at a note number as many as the distinct notes just
// before it, plus one: anything from 1 to min(position, m). Let each note
// take all it can while leaving one run for every later note.
var tune: [Note]Pitch = Pitch.first!
var left = k
for i in Note:
    let most = min(i.idx + 1, m)
    let take = min(most, left - (n - 1 - i.idx))
    if take < 1: break
    tune[i] = if take == most: Pitch.nth(i.idx % m) else: tune[Note.at(i - take)!]  // repeat the note `take` back
    left -= take
if left == 0: print(tune) else: print(-1)
