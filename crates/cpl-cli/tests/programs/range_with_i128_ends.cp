// `lo..hi` with I128 ends is a range value the VM builds itself.
let a: I128 = 5
let b: I128 = 9
let r = a..b
print(r.start, r.end)
