// spec/lib/search-dp.md indexes `[Group.subsets]` by `{}`; neither `{}` nor `Subset<G>.empty` is accepted.
domain G = 0..3
var best: [G.subsets]?Int = None
best[{}] = 0
best[Subset<G>.empty] = 0
print(best[Subset<G>.empty]!)
