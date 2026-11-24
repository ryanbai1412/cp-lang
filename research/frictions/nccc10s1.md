# Friction report: nccc10s1

## Dream

```cp
let answers: [Student, Question]Bool = read(as: 'T')
print(Subset<Question>.max(key => Student.min(s => Question.count(j => (j in key) == answers[s, j]))))
```

## Bitmasks become `Subset<Question>`

The v0.2 port packed each student's answers into an `Int` mask, counted wrong answers with a
hand-written `popcnt` loop and computed 2^k by repeated doubling. The rewrite iterates
`Subset<Question>` keys and counts matching answers directly.

## T/F as `Char`

The answers are read as `Char` and compared with `'T'` at each use. Reading a `T`/`F` grid
straight into `Bool` would type it at the read.

## `max`/`min` of non-empty generators are optional

There is always a key and a student, but both folds return `?Int` and need `!`.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`), the max case
passes, and corpus tests pass.
