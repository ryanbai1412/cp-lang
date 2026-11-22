Indexing a `List<T>` with an arbitrary `Int` inside nested loops emits a runtime bounds check on every iteration. Declare a `domain` and `[D]T` array so bounds are verified once.
