#include <cstdint>
#include <cstdio>
#include <vector>
int main() { long long n; if (scanf("%lld", &n) != 1) return 1; std::vector<uint32_t> out; for (long long i = 0; i < n; i++) out.push_back(i * 7919 % 536870912); for (size_t i = 0; i < out.size(); i++) { if (i) putchar(' '); printf("%u", out[i]); } putchar('\n'); }
