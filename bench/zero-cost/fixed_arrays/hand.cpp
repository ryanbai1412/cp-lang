#include <bits/stdc++.h>
using namespace std;

int64_t kernel(int n, int reps, const vector<int64_t>& a) {
    array<int64_t, 4> c{};
    for (int r = 0; r < reps; ++r) {
        for (int i = 0; i < n; ++i) {
            for (int k = 0; k < 4; ++k) c[k] = (c[k] * 31 + a[i] + k) % 1000000007;
        }
    }
    return c[0] + c[1] + c[2] + c[3];
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, reps;
    if (!(cin >> n >> reps)) return 0;
    vector<int64_t> a(n);
    for (int i = 0; i < n; ++i) cin >> a[i];
    cout << kernel(n, reps, a) << '\n';
}
