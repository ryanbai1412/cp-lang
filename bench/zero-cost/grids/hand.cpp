#include <bits/stdc++.h>
using namespace std;

int main() {
    int n, reps;
    if (!(cin >> n >> reps)) return 0;
    vector<vector<int64_t>> a(n, vector<int64_t>(n));
    for (int i = 0; i < n; ++i)
        for (int j = 0; j < n; ++j) a[i][j] = (i * 7 + j * 13) % 5 == 0 ? -1 : (i * 31 + j * 17) % 1000;
    int64_t acc = 0;
    for (int r = 0; r < reps; ++r) {
        vector<vector<int64_t>> c(n, vector<int64_t>(n, -1));
        for (int i = 0; i < n; ++i)
            for (int j = 0; j < n; ++j) {
                int64_t via = a[i][j];
                if (via < 0) continue;
                for (int l = 0; l < n; ++l) c[i][l] = max(c[i][l], via + a[j][l]);
            }
        acc = (acc + c[0][n - 1]) % 1000000007;
        a = move(c);
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
