// here: 4
// C++'s `sort(.., greater<int>())` (self-run mistakes): three errors, `let greater = todo`.
var a: List<Int> = [3, 1, 2]
sort(a.begin(), a.end(), greater<int>())
