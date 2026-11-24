// here: 4
// C++'s `greater<>()` (self-run mistakes): "expected a value after `<`", and a fix that removed only the `>`.
var a = [3, 1, 2]
sort(a.begin(), a.end(), greater<>())
