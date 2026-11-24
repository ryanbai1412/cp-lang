// Reference for Riolku's Mock CCC S2 (no accepted C++ on DMOJ; written for the corpus).
// Group the unordered index pairs {i, j} by a_i * a_j. Two different pairs with one product
// either are disjoint or share one index i with a_j == a_l; every disjoint pair of pairs gives
// 4 ordered tuples per order of the pairs.
#include <bits/stdc++.h>
using namespace std;

int main() {
    int n;
    cin >> n;
    vector<unsigned long long> a(n);
    for (auto &x : a) cin >> x;
    vector<unsigned long long> prod;
    prod.reserve((size_t)n * (n - 1) / 2);
    for (int i = 0; i < n; i++)
        for (int j = i + 1; j < n; j++) prod.push_back(a[i] * a[j]);
    sort(prod.begin(), prod.end());
    long long same = 0;
    for (size_t i = 0, j; i < prod.size(); i = j) {
        for (j = i; j < prod.size() && prod[j] == prod[i]; j++) {}
        long long c = j - i;
        same += c * (c - 1);
    }
    map<unsigned long long, long long> cnt;
    for (auto x : a) cnt[x]++;
    long long overlap = 0;
    for (auto x : a)
        for (auto [v, c] : cnt) {
            long long k = c - (v == x);
            overlap += k * (k - 1);
        }
    cout << 4 * (same - overlap) << "\n";
}
