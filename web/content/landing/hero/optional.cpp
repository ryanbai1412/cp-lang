#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    cin >> n;
    long long best = -1;
    for (int i = 0; i < n; i++) {
        long long x;
        cin >> x;
        if (x % 2 == 0) best = max(best, x);
    }
    cout << best << "\n";
}
