# ioi25_souvenirs

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port.

The port runs the dream's body behind the official manager's protocol over stdio: `transaction(m)` prints `? m` and reads the types bought and the change, and `interactor.py` judges it. It replaced a model-solution port that kept `lead_cost: [T]Int = -1` (an R5 sentinel) and `lead_bought` as parallel arrays, walked by two index cursors. It passes `cpl check`, `cpl lint`, the bounds check and its 12 package cases.

The dream's terms are:
- an `Eq` is a `Bitset<T>` of unknown types with their sum, kept per smallest unknown;
- prices are `[T]?Int`;
- `pay` and `learn` are the two moves the header describes.

## Checklist failures

- `for k in T if eq[k] is Some(e) and i in e.types: let e = mut eq[k]!` binds `e` twice, once to test and once to mutate. A mutable `if let` binding would bind it once.
- `while T.any(i => price[i] == None): for i in T.rev: ... break` restarts the right-to-left scan after every move. That is O(N^2) per transaction for N = 100, which is fine, but a "next move" function returning the action would state the loop's shape.

- `for i in T: repeat i.ord - bought[i]: transaction(price[i]!)` unwraps every price after the loop proves them all known; the type does not carry that, so the end needs `!`.

## Gaps that need the language

- The `@grader` interface and `break` with a value: research/frictions/ioi25_souvenirs.md.
