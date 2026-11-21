#include <bits/stdc++.h>
using namespace std;

int main() {
    const int n = 100000;
    vector<long long> base(n);
    for (int i = 0; i < n; i++) base[i] = (1LL * i * 37 + 13) % 1000;
    vector<vector<long long>> lv;
    for (int half = 1; half < n; half *= 2) {
        vector<long long> row = base;
        for (int mid = half; mid < n; mid += 2 * half) {
            for (int i = mid - 2; i >= mid - half; i--) row[i] = base[i] + row[i + 1];
            int hi = min(mid + half, n);
            for (int j = mid + 1; j < hi; j++) row[j] = row[j - 1] + base[j];
        }
        lv.push_back(move(row));
    }
    long long s = 0, x = 9876543;
    for (int i = 0; i < 500000; i++) {
        x = (x * 1103515245LL + 12345) & 2147483647LL;
        int l = x % n;
        x = (x * 1103515245LL + 12345) & 2147483647LL;
        int r = x % n;
        int lo = min(l, r), hi = max(l, r);
        if (lo == hi) s += base[lo];
        else {
            int k = 31 - __builtin_clz(lo ^ hi);
            s += lv[k][lo] + lv[k][hi];
        }
    }
    cout << s << "\n";
}
