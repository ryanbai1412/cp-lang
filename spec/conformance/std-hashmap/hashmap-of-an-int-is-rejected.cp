// expect: E0302 at 2
// `HashMap(5)` is not a capacity (C++'s bucket count) or anything else, so it is an error rather than an empty map.
let m: HashMap<Int, Int> = HashMap(5)
