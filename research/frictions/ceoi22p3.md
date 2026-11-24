# ceoi22p3 frictions

## Dream

The committed `dream.cp` is the target; the port is it with today's `Rooted.from_parents`
spelling and `picked.adjacent.collect()` for the asked pairs.

## Interactive verification

The package is judged by its interactor; there is no accepted C++ to compare against and the old
port is only judged the same way.

## The interactor's own speed counts against the case

`cpl test` stops a case once its wall clock passes max(limit, 10 s), and that includes the
interactor. One LCA walk per pair took longer than that on 1e5-vertex trees, so `interactor.py`
answers every pair at once with an offline Tarjan walk.
