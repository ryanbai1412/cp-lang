#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void solve() {
    int n, kv, ql, qr;
    cin >> n >> kv >> ql >> qr;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    vector<long> carr;
    for (int i = 0; i < n; i++) {
        long csum = 0;
        for (int j = i; j < n; j++) {
            csum += arr[j];
            if (ql <= j - i + 1 && j - i + 1 <= qr) {
                carr.push_back(csum);
            }
        }
    }

    partial_sort(carr.begin(), carr.begin() + kv, carr.end(), greater<>());
    cout << accumulate(carr.begin(), carr.begin() + kv, long(0)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}