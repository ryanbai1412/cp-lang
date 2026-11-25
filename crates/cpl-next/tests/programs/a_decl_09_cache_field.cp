// expect: ok, output "1"
// rule: R02.6
// A @cache field is skipped by ==; here only that the decorator is not silently dropped.
struct D:
    var p: List<Int>
    @cache var hits: Int
var d = D([1], 1)
print(d.hits)
