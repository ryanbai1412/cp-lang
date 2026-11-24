#include <cstdio>
#include <vector>

std::vector<int> max_coupons(int A, std::vector<int> P, std::vector<int> T);

int main() {
  std::vector<int> R = max_coupons(13, {4, 500, 8, 14, 3}, {1, 3, 3, 4, 2});
  printf("%d\n", (int)R.size());
  for (int i = 0; i < (int)R.size(); i++) printf("%s%d", i ? " " : "", R[i]);
  printf("\n");
}
