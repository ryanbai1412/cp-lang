// expect: E0303 at 2
// `SortedMap(pairs)` is an error, not an empty map.
let m: SortedMap<Int, Int> = SortedMap([(1, 2)])
