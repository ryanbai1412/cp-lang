// expect: E0302 at 3
// A list that isn't `(key, value)` pairs is not a map's contents, so `HashMap(a)` is an error rather than an empty map.
let a: List<Int> = [3, 1, 3]
let m = HashMap(a)
