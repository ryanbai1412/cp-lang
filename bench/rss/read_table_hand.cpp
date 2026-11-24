#include <cstdint>
#include <cstdio>
#include <vector>
int main() { int r, c; if (scanf("%d %d", &r, &c) != 2) return 1; std::vector<uint16_t> oil((size_t)r * c); for (auto& x : oil) { unsigned v; if (scanf("%u", &v) != 1) return 1; x = (uint16_t)v; } long long s = 0; for (int i = 0; i < r; i++) for (int j = 0; j < c; j++) s += (long long)oil[(size_t)i * c + j] * (i + j); printf("%lld\n", s); }
