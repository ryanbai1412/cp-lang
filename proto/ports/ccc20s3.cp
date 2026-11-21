// CCC 2020 S3 Searching for Strings: count the distinct substrings of the
// haystack that are rearrangements of the needle.
use std.strings.{HashVal}

domain Letter = 'a'..='z'
let needle: Str = read()
let hay: Str = read()

if needle.len > hay.len:
    print(0)
    return

var missing: [Letter]Int = 0
var nonzero = 0
for c in needle:
    let k = Letter(c)
    if missing[k] == 0: nonzero += 1
    missing[k] += 1

let rh = RollingHash.new(hay)
var found = Set<HashVal>()
for i in 0..hay.len:
    let add = Letter(hay[i])
    if missing[add] == 0: nonzero += 1
    missing[add] -= 1
    if missing[add] == 0: nonzero -= 1
    if i >= needle.len:
        let drop = Letter(hay[i - needle.len])
        if missing[drop] == 0: nonzero += 1
        missing[drop] += 1
        if missing[drop] == 0: nonzero -= 1
    if i + 1 >= needle.len and nonzero == 0:
        found.insert(rh.slice(i + 1 - needle.len, i + 1))
print(found.len)
