// expect: E0303
// `SortedMap(pairs)` is an error, not an empty map.
let m: SortedMap<Int, Int> = SortedMap([(1, 2)]) // <- here
