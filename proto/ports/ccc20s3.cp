// cp: 0.1 6ac88e2+dirty
// CCC 2020 S3 Searching for Strings: count the distinct substrings of the
// haystack that are rearrangements of the needle.
domain Letter = 'a'..='z'
let needle: [domain N]Letter = read()
let hay: [domain H]Letter = read()

// Slide a window of the needle's length over the haystack. `missing` is the
// needle's letters minus the window's: the window is a rearrangement exactly
// when every count in it is zero.
var missing = counts(needle)
let hash = StrHash(hay)
var found: Set<Int> = []
for i in H:
    missing[hay[i]] -= 1
    if let gone = i - N.len: missing[hay[gone]] += 1
    if missing.distinct == 0: found.insert(hash.of((..=i).suffix(N.len)))
print(found.len)
