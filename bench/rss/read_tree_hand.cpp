#include <cstdio>
#include <tuple>
#include <vector>
int main() {
  int n, k;
  if (scanf("%d %d", &n, &k) != 2) return 1;
  std::vector<std::tuple<int, int, long long>> hills(n - 1);
  for (auto& [u, v, d] : hills) if (scanf("%d %d %lld", &u, &v, &d) != 3) return 1;
  std::vector<long long> skills(k);
  for (auto& s : skills) if (scanf("%lld", &s) != 1) return 1;
  long long total = 0;
  for (auto& h : hills) total += std::get<2>(h);
  for (long long s : skills) total += s;
  printf("%lld\n", total);
}
