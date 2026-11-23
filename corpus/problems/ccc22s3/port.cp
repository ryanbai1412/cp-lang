// CCC 2022 S3 Good Samples: write n notes with pitches 1..=m so that exactly
// k runs of consecutive notes have all pitches different, or report -1.
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
    tune[i] = if take == most: Pitch.nth(i.idx % m) else: tune[(i - take)!]  // repeat the note `take` back
    left -= take
if left == 0: print(tune) else: print(-1)
