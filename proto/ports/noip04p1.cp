// NOIP 2004 Jinjin's Savings Plan: each month she gets 300, spends her
// budget and banks every whole hundred left over; the bank pays 20% at the
// end of the year. Print minus the first month she runs short, or her money
// at the end.
var cash = 0
var banked = 0
var short: ?Int = none                  // the first month the budget does not fit
for month in 1..=12:
    let budget: 0..=350 = read()
    if short == none:
        cash += 300 - budget
        if cash < 0: short = month
        else:
            banked += cash / 100 * 100
            cash %= 100
if let month = short: print(-month) else: print(cash + banked * 6 / 5)
