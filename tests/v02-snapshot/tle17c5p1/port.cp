// Source: TLE '17 Contest 5 P1 (Guess the Number), https://dmoj.ca/problem/tle17c5p1
// Problem: interactive. A secret N in -50..=50; `guess n` (n in -50..=50) replies |N - n|.
//   Print `answer N` after as few guesses as possible.
// Solution: guess -50: since N >= -50 the reply is N + 50, so one guess gives N = reply - 50.
// original: corpus/submissions/5034414.cpp; judge: tle17c5p1_judge.py
let diff: -2e9..=2e9 = ask("guess", -50)
print("answer", diff - 50)
