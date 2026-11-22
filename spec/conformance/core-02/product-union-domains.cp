// expect: ok, output "6 5"
// rule: 02.9.6
domain V = 0..2
domain K = 0..3
domain S = (V, K)
domain N = V + K
print(S.count, N.count)
