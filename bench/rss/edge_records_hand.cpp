#include <cstdint>
#include <cstdio>
#include <vector>
struct E { uint32_t u, v; uint16_t w; };
int main() { int n; if (scanf("%d", &n) != 1) return 1; std::vector<E> roads(n); for (auto& e : roads) { unsigned u, v, w; if (scanf("%u %u %u", &u, &v, &w) != 3) return 1; e = {u, v, (uint16_t)w}; } long long s = 0; for (auto& e : roads) s += 3LL * e.u + e.v + e.w; printf("%lld\n", s); }
