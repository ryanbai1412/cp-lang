#include <bits/stdc++.h>
using namespace std;

int64_t kernel(int n, int reps, const vector<int64_t>& a) {
    vector<int64_t> arr(n);
    for (int i = 0; i < n; ++i) arr[i] = a[i];
    int64_t acc = 0;
    for (int r = 0; r < reps; ++r) {
        for (int i = 0; i < n; ++i) {
            acc = (acc + arr[i] * (int64_t)(i + 1)) ^ (acc >> 3);
        }
    }
    return acc;
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
