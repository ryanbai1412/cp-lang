# Friction report: dmopc20c6p6

No semantic gap. The old port judged itself with an inline grader over the hidden capacities; it now talks to the judge through `Wire`, and `interactor.py` (the statement's 1e5-operation limit included) judges it. N >= 2 is a type: one bucket can never be measured.

## Measurements

gen.py --max (N = 4e4, M = 1e4): port 1.38 s, accepted C++ 1.37 s, both with the Python judge in the loop and within 1e5 operations.
