#include <cstdio>
#include <vector>

std::vector<int> f(std::vector<int> P);

int main() {
  std::vector<int> R = f({5, 6, 7});
  printf("%d %d\n", R[0], R[1]);
}
