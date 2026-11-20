#pragma GCC optimize("Ofast","unroll-loops")
#include "bits/extc++.h"

using namespace std;

const int maxn = 65534;

int main() {
  uint32_t ans = 0;
#pragma GCC unroll maxn
  for(int i = 0; i < maxn; i++) {
#pragma GCC unroll maxn
    for(int j = 0; j < maxn; j++) {
      ans += rand() * rand();
    }
  }
  cout << ans << endl;
}