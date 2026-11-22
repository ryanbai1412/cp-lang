#include <bits/stdc++.h>
using namespace std;

int64_t kernel(int n, int reps, const vector<int64_t>& a) {
    int64_t total = 0;
    for (int r = 0; r < reps; ++r) {
        int64_t s = 0;
        for (int i = 0; i < n; ++i) {
            int64_t x = a[i];
            if (((x + r) & 1) == 0) {
                s += (x + r) * 3 + 1;
            }
        }
        total += s;
    }
    return total;
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
