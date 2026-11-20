#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n), tmp(n);
    for (auto& a : arr) {
        cin >> a;
    }
    sort(begin(arr), end(arr), greater<>());
    while (q--) {
        int ind, x;
        cin >> x >> ind;
        for (int i = 0; i < ind; i++) {
            arr[i] -= x;
        }
        merge(arr.begin(), arr.begin() + ind, arr.begin() + ind, arr.end(),
              tmp.begin(), greater<>());
        swap(arr, tmp);
    }
    for (int i = 0; i < n; i++) {
        cout << arr[i] << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}