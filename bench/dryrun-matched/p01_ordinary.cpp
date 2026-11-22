#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int t;
    cin >> t;
    while (t--) {
        long long n;
        cin >> n;
        int c = 0;
        for (int d = 1; d <= 9; d++)
            for (long long x = d; x <= n; x = x * 10 + d) c++;
        cout << c << '\n';
    }
}
