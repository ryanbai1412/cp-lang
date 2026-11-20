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
    vector<int> arr;
    {
        int x;
        while (cin >> x) {
            arr.push_back(x);
        }
    }
    int n = sz(arr);
    {
        int dp[n] {};
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < i; j++) {
                if (arr[j] >= arr[i]) {
                    dp[i] = max(dp[i], dp[j]);
                }
            }
            dp[i]++;
        }
        cout << *max_element(dp, dp + n) << endl;
    }
    multiset<int> s;
    for (auto& a : arr) {
        auto it = s.lower_bound(a);
        if (it == s.end()) {
            s.insert(a);
        } else {
            it = s.erase(it);
            s.insert(it, a);
        }
    }
    cout << sz(s) << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    solve();
}