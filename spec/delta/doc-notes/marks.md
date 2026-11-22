# marks doc notes

- `MarkMap.get`: contradiction. `spec/lib/containers.md` §2.7 has `get(self, v) -> T` returning the default when unset; the stub has `-> ?T`. Followed the stub: absent reads `None`, and `default_val` only fills storage.
- `MarkMap.remove`: contradiction. The library prose has `remove -> Bool`; the stub has `-> ?T` returning the old value. Followed the stub.
- `MarkMap.new`: the library prose names the parameter `default`; the stub names it `default_val`. Followed the stub.
- `MarkMap.contains`: chose that a key explicitly set to `default_val` is present (presence is by stamp, not by value).
- `Interner.id`, `Interner.get`: contradiction on cost. The library prose says O(log n); the stub says O(1) expected (it is backed by `HashMap`). Followed the stub.
- `Interner`: the library prose lists `@get keys -> List<K>`; the stub has no `keys` but adds `clear`, `contains` and `reserve`. Documented the stub as is.
- `Interner.clear`: the stub says O(1); clearing a `HashMap` is normally O(capacity). Left the stated cost; flag for the implementer.
- `Interner.key`, `Arena.get`/`set`: chose a trap for an id or handle outside `0..len`.
- `Marks.clear`, `MarkMap.clear`: chose "reset the arrays on the call that would push the generation past `2_000_000_000`" and documented the cost as O(1) amortized with that one O(|D|) call.
- `SparseSet.each`: the stub says "arbitrary order", which two implementations can resolve differently. Chose the Briggs-Torczon dense order (insertion order, with `remove` moving the last element into the hole) so results are deterministic.
- `Arena.get`: a handle from before `clear` that is still below `len` cannot be detected without a generation tag; documented that it reads the slot's current node.
- `Arena.alloc`: chose a trap past `2_000_000_001` nodes, the range of `Id.raw`.
- `Arena.new`, `Arena.reserve`, `Interner.reserve`: chose a trap on a negative `cap`.
