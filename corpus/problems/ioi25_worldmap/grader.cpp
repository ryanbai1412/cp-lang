// IOI 2025 World Map sample grader (task archive graders/grader.cpp, without the
// judge-only secret checks): T scenarios of `N M` and M pairs `A B`; prints each map
// as P, the row lengths, a blank line and the rows, scenarios separated by a blank line.
#include "worldmap.h"
#include <cassert>
#include <cstdio>

int main() {
  int T;
  assert(1 == scanf("%d", &T));
  std::vector<int> Nt(T), Mt(T);
  std::vector<std::pair<std::vector<int>, std::vector<int>>> AB;
  for (int t = 0; t < T; ++t) {
    assert(2 == scanf("%d %d", &Nt[t], &Mt[t]));
    std::vector<int> A(Mt[t]), B(Mt[t]);
    for (int i = 0; i < Mt[t]; i++) assert(2 == scanf("%d %d", &A[i], &B[i]));
    AB.push_back(make_pair(A, B));
  }
  std::vector<std::vector<std::vector<int>>> Ct;
  for (int t = 0; t < T; t++) Ct.push_back(create_map(Nt[t], Mt[t], AB[t].first, AB[t].second));
  for (int t = 0; t < T; t++) {
    auto& C = Ct[t];
    int P = (int)C.size();
    printf("%d\n", P);
    for (int i = 0; i < P; ++i) printf("%d%c", (int)C[i].size(), " \n"[i + 1 == P]);
    printf("\n");
    for (int i = 0; i < P; i++) {
      int Q = (int)C[i].size();
      for (int j = 0; j < Q; j++) printf("%d%c", C[i][j], " \n"[j + 1 == Q]);
    }
    if (t < T - 1) printf("\n");
  }
  return 0;
}
