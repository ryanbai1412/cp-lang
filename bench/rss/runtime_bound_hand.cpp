#include <cstdint>
#include <cstdio>
#include <vector>
int main() { int n; if (scanf("%d", &n) != 1) return 1; std::vector<uint32_t> walk; for (int i = 0; i < n; i++) walk.push_back(n - i); std::vector<uint32_t> nxt(n); for (int i = 0; i < n; i++) nxt[i] = i + 1; long long s = 0; for (int i = 0; i < n; i++) s += 2LL * walk[i] + nxt[i]; printf("%lld\n", s); }
