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

void solve1(int n, int s, int c) {
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

void solve2(int n, int s, int c) {
    vector<int> ans;
    int l1 = 1, r1 = n / 2, l2 = n / 2 + 1, r2 = n;
    while (l1 <= r1 || l2 <= r2) {
        if (r1 - l1 >= r2 - l2) {
            ans.push_back(r1);
            r1--;
        } else {
            ans.push_back(l2);
            l2++;
        }
        if (sz(ans) % (s + c) < s) {
            if (r1 > 1) {
                r1--;
            }
            if (l2 < n) {
                l2++;
            }
            l1 = max(1, l1 - 1);
            r2 = min(n, r2 + 1);
        } else {
            r1++;
            l1++;
            r2--;
            l2--;
        }
        dbg(l1, r1, l2, r2);
    }
    answer(ans);
}

void solve() {
    int n, s, c;
    cin >> n >> s >> c;
    if (s <= c) {
        solve1(n, s, c);
    } else {
        solve2(n, s, c);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}