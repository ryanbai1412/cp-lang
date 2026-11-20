// NOIP 2004 Jinjin's Savings Plan: each month she gets 300, spends her
// budget and banks every whole hundred left over; the bank pays 20% at the
// end of the year. Print minus the first month she runs short, or her money
// at the end.
let budgets: [0..12]0..=350 = read()

var (cash, banked) = (0, 0)
for month in 0..12:
    cash += 300 - budgets[month]
    if cash < 0:
        print(-(month + 1))
        return
    banked += cash / 100 * 100
    cash %= 100
print(cash + banked * 6 / 5)
