#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    cin >> n;
    assert(1 <= n && n <= 200000);
    vector<int> a(n);
    long long sum = 0;
    for (int& x : a) {
        cin >> x;
        assert(1 <= x && x <= 100);
        sum += x;
    }
    cout << sum << "\n";
}
