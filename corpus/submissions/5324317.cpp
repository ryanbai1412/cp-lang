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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

bool cmp(const pair<int, int>& a, const pair<int, int>& b) {
    return a.first + a.second < b.first + b.second;
}

pair<int, int> solve(const vector<int>& arra, const vector<int>& arrb) {
    int n = sz(arra);
    int sa = accumulate(begin(arra), end(arra), 0),
        sb = accumulate(begin(arrb), end(arrb), 0);
    pair<int, int> ans {INT_MAX, 0};
    for (int i = 0; i < (1 << n); i++) {
        int asum = 0, bmn = 1e9;
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                asum += arra[j];
                bmn = min(bmn, arrb[j]);
            }
        }
        if (asum && bmn) {
            ans = min(ans, {sa - asum + 1, sb - bmn + 1}, cmp);
        }
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr[2];
    for (auto& a : arr) {
        a.resize(n);
        for (auto& b : a) {
            cin >> b;
        }
    }
    auto [a, b] = min(solve(arr[0], arr[1]), solve(arr[1], arr[0]), cmp);
    cout << a << endl;
    cout << b << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}