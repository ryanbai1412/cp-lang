// expect: ok, output "50"
// gap: compiler 2026-11-24T05Z
// rule: 04a.9.6
// `keyed(by:)` holds the key fn, so `argmax` answers with `by(x)` of the largest element.
let k = [3, 5].keyed(by: x => x * 10)
print(k.argmax()!)
