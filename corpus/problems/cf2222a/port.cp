// Source: Codeforces 2222A (A Wonderful Contest), https://codeforces.com/contest/2222/problem/A
// limits: 1000 ms, 256 MB
// Problem: n <= 10 problems; problem i has a[i] subtasks (a[i] divides 100), each worth
//   100 / a[i]. Print Yes if every total score 0..=100n is reachable, else No. Multitest.
// Solution: a total of 1 needs a subtask worth 1, so some a[i] = 100; with one, its 100 unit
//   subtasks make any remainder while every other problem scores 0 or 100. Check whether
//   some a[i] == 100, O(n).

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=10 ** 9 = read()
print(yes(a.any(x => x == 100), answers: ("Yes", "No")))
