// Source: Mock CCC '22 Contest 2 S1 (Kaguya Wants to Let Ishigami Pass),
//   https://dmoj.ca/problem/nccc10s1
// Problem: n <= 1000 students' answers to k <= 10 true/false questions. Choose the answer key
//   that maximises the lowest score; print that score.
// Solution: try all 2^k keys; a key's value is the lowest score any student gets on it.
//   Print the best. O(2^k n k).
// Bounds: n: 1..=1_000; k: 1..=10
// original: corpus/submissions/4214147.cpp
let (n, k): (1..=1_000, 1..=10) = read()
domain Student = 0..n
domain Question = 0..k
let answers: [Student, Question]Char = read()

fn score(key: Subset<Question>, s: Student) -> Int =
    Question.count(j => (j in key) == (answers[s, j] == 'T'))
print(max(min(score(key, s) for s in Student)! for key in Subset<Question>)!)
