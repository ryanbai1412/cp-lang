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

namespace s1 {

bool solve(vector<int> arr, const vector<pair<int, int>>& queries) {
    if (!(sz(arr) <= 50000 && sz(queries) <= 100)) {
        return false;
    }
    int n = sz(arr);
    for (auto& [ind, x] : queries) {
        sort(begin(arr), end(arr), greater<>());
        for (int i = 0; i < ind; i++) {
            arr[i] -= x;
        }
    }
    sort(begin(arr), end(arr), greater<>());
    for (int i = 0; i < n; i++) {
        cout << arr[i] << " \n"[i == n - 1];
    }
    return true;
}

}

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    vector<pair<int, int>> queries(q);
    for (auto& [ind, x] : queries) {
        cin >> ind >> x;
    }
    s1::solve(arr, queries);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}