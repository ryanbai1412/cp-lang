// cp: 0.1 18d1f9f+dirty
// CCC 2020 S3 Searching for Strings: count the distinct substrings of the
// haystack that are rearrangements of the needle.
let needle: [domain N]Char = read()
let hay: [domain H]Char = read()
domain Letter = 'a'..='z'

var want: [Letter]Int = 0
for c in needle: want[Letter(c)] += 1

// Slide a window of the needle's length; `off` counts letters whose count in
// the window differs from the needle's.
var have: [Letter]Int = 0
var off = count(want[c] != 0 for c in Letter)
fn shift(c: Letter, by: Int):
    if have[c] == want[c]: off += 1
    have[c] += by
    if have[c] == want[c]: off -= 1

let hash = StrHash(hay)
var found: Set<Int> = []
for end in H:
    shift(Letter(hay[end]), 1)
    if let start = H.try_at(end.idx - N.len + 1):
        if off == 0: found.insert(hash.of(start, end))
        shift(Letter(hay[start]), -1)
print(found.len)
