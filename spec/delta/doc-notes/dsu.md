# dsu: doc-standard notes

- All union-finds (`Dsu`, `DsuOf`, `ParityDsu`, `PotentialDsu`, `PersistentDsu`): which root survives a merge of equal-size sets was unspecified, and it is observable through `find`, `merge`, `roots` and `values`. Chose `a`'s root (the first argument), the larger set's root otherwise.
- `Dsu.roots`, `DsuOf.roots`, `DsuOf.values`: order unspecified; chose increasing positions of the representative, so `values` lines up with `roots`.
- `Dsu.groups`: order unspecified; chose lists by their least position, positions increasing within each (first in key order).
- `undo_to` (all five): chose to trap when the mark lies after the present (its changes were already undone); a `Mark` from another structure cannot be detected and is not promised to trap.
- `ParityDsu.unite`, `PotentialDsu.unite`: a contradicting constraint merges nothing and only counts the contradiction; an already-implied constraint returns `true` and logs nothing. Follows "false if that contradicts" plus the log comment in the struct.
- `ParityDsu.consistent`: "every `unite` so far agreed" read as "no contradiction is in effect", since `undo_to` takes contradictions back.
- `PersistentDsu.unite`: chose handles numbered in creation order (k-th call returns k) and a fresh handle even when nothing merges, so a handle count always equals the number of calls.
- `PersistentDsu.find/same/size/unite`: chose to trap on a `ver` that is not a handle of this structure.
- `PersistentDsu.reserve`, `RollbackArray`: trap on `nodes < 0`; `RollbackArray.set` logs every write, marked or not (the struct has no marked flag).
- `RollbackArray.new` (contradiction): `spec/lib/containers.md` declares `new(init: [D]T)` and a `filled(val: T)` constructor; the stub has `new(init: D -> T)` and no `filled`. Left unchanged.
- `RollbackArray.new` (contradiction): the old doc and `containers.md` give cost `O(D.K)`, an undefined name; documented O(n) for n = `D.count`.
- `DsuOf.groups` (contradiction): `containers.md` asks for `groups()` on `DsuOf` to match `Dsu`; the stub has none. Left unchanged.
- `PotentialDsu.new` (contradiction): its doc carries a `[rename]` marker and a generic `op: ?G` group operation, but `find`, `unite` and `diff` take and return `Int` offsets, so a non-`Int` group cannot be expressed. Left the signature; documented `+` on `Int` as the default.
- `Dsu.find` / `DsuOf.find` take `self` while `same`/`size` take `mut self` although all shorten paths through `@cache`; harmless, left unchanged.
