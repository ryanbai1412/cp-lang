A `Map<V, T>` or `Set<V>` keyed by a dense domain `V` pays hashing or tree overhead on every lookup. Use a flat `[V]?T` or `[V]Bool` array instead.
