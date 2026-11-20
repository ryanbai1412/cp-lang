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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "{" << p.first << ", " << p.second << "}";
}

long min_dist1(vector<int> arr) {
    sort(begin(arr), end(arr));
    long ans = 0;
    for (auto& a : arr) {
        ans += abs(a - arr[sz(arr) / 2]);
    }
    return ans;
}

vector<int> map_first(const vector<pair<int, int>>& arr) {
    vector<int> ans;
    for (auto& a : arr) {
        ans.push_back(a.first);
    }
    return ans;
}

struct DS {
    long sums[2] {};
    multiset<int> s[2];

    void insert(int ind, int x) {
        s[ind].insert(x);
        sums[ind] += x;
    }

    void erase(int ind, int x) {
        s[ind].erase(s[ind].find(x));
        sums[ind] -= x;
    }

    int first(int ind) const {
        return *s[ind].begin();
    }

    int last(int ind) const {
        return *(--s[ind].end());
    }

    void insert(int x) {
        if (!sz(s[0])) {
            insert(0, x);
            return;
        }
        if (x <= last(0)) {
            insert(0, x);
        } else {
            insert(1, x);
        }
        if (sz(s[1]) > sz(s[0])) {
            int cx = first(1);
            erase(1, cx);
            insert(0, cx);
        } else if (sz(s[0]) - sz(s[1]) == 2) {
            int cx = last(0);
            erase(0, cx);
            insert(1, cx);
        }
    }

    long query() const {
        if (!sz(s[0])) {
            return 0;
        }
        long med = last(0);
        return med * sz(s[0]) - sums[0] + sums[1] - med * sz(s[1]);
    }
};

long solve(vector<pair<int, int>> arr) {
    int n = sz(arr);
    sort(begin(arr), end(arr));
    long ans = min_dist1(map_first(arr));

    long ql[n + 1], qr[n + 1];
    {
        DS ds;
        for (int i = 0; i <= n; i++) {
            ql[i] = ds.query();
            if (i < n) {
                ds.insert(arr[i].first);
            }
        }
    }
    {
        DS ds;
        for (int i = n; i >= 0; i--) {
            if (i < n) {
                ds.insert(arr[i].first);
            }
            qr[i] = ds.query();
        }
    }

    DS dsl, dsr;
    int cadd = 0;
    for (auto& [a, b] : arr) {
        dsr.insert(a);
        cadd += !b;
    }
    for (int i = 0; i <= sz(arr); i++) {
        ans = min(ans, ql[i] + qr[i] + cadd);
        if (i < sz(arr)) {
            auto& [a, b] = arr[i];
            cadd += b - !b;
        }
    }
    dbg(arr, ans);
    return ans;
}

void solve() {
    int n;
    cin >> n;
    int arr[n][3];
    for (auto& [x, y, v] : arr) {
        cin >> x >> y >> v;
    }
    long ans = 0;
    {
        vector<pair<int, int>> vx;
        for (auto& [x, _y, v] : arr) {
            if (v == 0 || v == 1) {
                vx.emplace_back(x, 0);
            } else {
                vx.emplace_back(x, 1);
            }
        }
        ans += solve(vx);
    }
    {
        vector<pair<int, int>> vy;
        for (auto& [_x, y, v] : arr) {
            if (v == 0 || v == 3) {
                vy.emplace_back(y, 0);
            } else {
                vy.emplace_back(y, 1);
            }
        }
        ans += solve(vy);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}