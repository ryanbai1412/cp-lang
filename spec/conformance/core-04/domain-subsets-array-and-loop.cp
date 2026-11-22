// expect: ok, output "4 7"
// rule: 04.4.1

domain P = [2]
var dp: [P.subsets]Int = 0
dp[Subset<P>.empty] = 1
var count = 0
for s in P.subsets:
    count += 1
dp[Subset<P>.full] = 7
print(count, dp[Subset<P>.full])
