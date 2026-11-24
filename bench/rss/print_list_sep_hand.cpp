#include <cstdio>
#include <vector>
int main() { int n; if (scanf("%d", &n) != 1) return 1; std::vector<long long> xs(n); for (int i = 0; i < n; i++) xs[i] = 7LL * i - n; for (int i = 0; i < n; i++) printf("%lld\n", xs[i]); }
