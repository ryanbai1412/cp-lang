// expect: ok, output "301 402 501"
// rule: 04b.5.3
print("xxxabab"[3..6].prefix_matches("ab").items.map((k, x) => k * 100 + x).collect())
