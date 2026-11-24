#include <cstdint>
#include <cstdio>
#include <utility>
#include <vector>
struct E { int u, v; uint32_t d; };
int main() { int n; if (scanf("%d", &n) != 1) return 1; std::vector<E> hills(n); for (auto& e : hills) { unsigned d; if (scanf("%d %d %u", &e.u, &e.v, &d) != 3) return 1; e.d = d; } std::vector<std::vector<std::pair<int, uint32_t>>> adj(n); for (auto& e : hills) { adj[e.u - 1].push_back({e.v, e.d}); adj[e.v - 1].push_back({e.u, e.d}); } long long s = 0; for (auto& row : adj) for (auto& [to, d] : row) s += (to - 1) + d; printf("%lld\n", s); }
