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

void answer(const vector<int>& ans) {
    int n = sz(ans);
    cout << n << endl;
    for (int i = 0; i < n; i++) {
        cout << ans[i] << " \n"[i == n - 1];
    }
}

void solve() {
    int n, s, c;
    cin >> n >> s >> c;
    if (!c) {
        vector<int> ans;
        int r = n / 2, l = n / 2 + 1;
        while (1 <= r || l <= n) {
            if (r - 1 >= n - l) {
                ans.push_back(r);
                r--;
            } else {
                ans.push_back(l);
                l++;
            }
            if (r > 1) {
                r--;
            }
            if (l < n) {
                l++;
            }
        }
        answer(ans);
        return;
    }
    assert(s <= c);
    vector<int> ans;
    int l = 1, r = n;
    while (l < r) {
        if (sz(ans) % (s + c) < s) {
            ans.push_back(r);
            if (r - l + 1 == 2) {
                r = l = max(1, l - 1);
            } else {
                l = max(1, l - 1);
                r -= 2;
            }
        } else {
            ans.push_back(l + 1);
            if (r - l + 1 == 2) {
                l++;
            } else {
                l++;
                r--;
            }
        }
    }
    if (l == r) {
        ans.push_back(l);
    }
    answer(ans);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}