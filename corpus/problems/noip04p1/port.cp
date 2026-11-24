// Source: NOIP 2004 Senior P1 (Jinjin's Savings Plan), https://dmoj.ca/problem/noip04p1
// Problem: each month adds 300 to Jinjin's cash and she spends that month's budget (12 budgets,
//   each <= 350); every whole hundred left is deposited with mom, who returns deposits +20% at
//   year end. Print -X for the first month X whose budget exceeds her cash, else her final money.
// Solution: simulate the 12 months: cash += 300 - budget; the first negative cash ends it,
//   else bank cash / 100 * 100 and keep cash % 100. Answer cash + banked * 6 / 5.
// original: corpus/submissions/5117097.cpp
domain Month = 1..=12
let budget: [Month]0..=350 = read()

var cash = 0
var banked = 0
var short: ?Month = None  // the first month the budget does not fit
for month in Month:
    cash += 300 - budget[month]
    if cash < 0:
        short = month
        break
    banked += cash / 100 * 100
    cash %= 100
if let month = short: print(-Int(month)) else: print(cash + banked * 6 / 5)
