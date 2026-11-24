// The judge's grader: reads a list, calls `total`, prints what it returns.
#include "sum.h"
#include <cstdio>

int main() {
  int n;
  if (std::scanf("%d", &n) != 1) return 1;
  std::vector<int> a(n);
  for (int &x : a)
    if (std::scanf("%d", &x) != 1) return 1;
  std::printf("%d\n", total(a));
  return 0;
}
