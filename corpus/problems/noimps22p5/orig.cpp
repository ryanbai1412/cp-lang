// Accepted submission 5448138, plus the x = y = 0 branch it lacks (it printed nothing there;
// the answer is 0: with no costs every arrangement is free).
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
#define sz(x) int((x).size())

struct DS1 {
    long sum = 0;
    multiset<int> s;

    void insert(int x) {
        sum += x;
        s.insert(x);
    }

    void erase(int x) {
        sum -= x;
        s.erase(s.find(x));
    }

    int size() const {
        return sz(s);
    }

    int first() const {
        return *s.begin();
    }

    int first2() const {
        return *(++s.begin());
    }

    int last() const {
        return *(--s.end());
    }
};

constexpr int maxn = 4e5 + 5;

vector<int> graph[maxn], arr[maxn];
int n, iarr[maxn];

void dfs(int u, int d) {
    arr[d].push_back(iarr[u]);
    for (auto& v : graph[u]) {
        dfs(v, d + 1);
    }
}

void solve01() {
    DS1 ds;
    long ans = 0;
    for (int i = 0; i < n; i++) {
        for (auto& a : arr[i]) {
            ds.insert(a);
        }
        ds.erase(ds.last());
        ans += ds.sum;
    }
    cout << ans << endl;
}

void solve11() {
    DS1 ds;
    long ans = 0;
    for (int i = 0; i < n; i++) {
        for (auto& a : arr[i]) {
            ds.insert(a);
        }
        int m = sz(ds);
        ans += ds.sum + long(m - 2) * ds.first();
        ds.erase(ds.last());
    }
    cout << ans << endl;
}

void solve10() {
    auto go = [&](bool pmin) -> long {
        DS1 ds;
        long ans = 0;
        for (int i = 0; i < n; i++) {
            for (auto& a : arr[i]) {
                ds.insert(a);
            }
            if (i == n - 2 && sz(ds) == 2) {
                int first = ds.first();
                ans += first;
                ds.erase(first);
                continue;
            }
            int m = sz(ds);
            if (m == 1) {
                ds.erase(ds.first());
            } else if (m == 2) {
                if (pmin) {
                    int first = ds.first();
                    ans += first;
                    ds.erase(first);
                } else {
                    int last = ds.last();
                    ans += last;
                    ds.erase(last);
                }
            } else {
                int first2 = ds.first2();
                ans += long(m - 2) * ds.first() + first2;
                ds.erase(first2);
            }
        }
        return ans;
    };
    cout << min(go(false), go(true)) << endl;
}

void solve() {
    int x, y;
    cin >> n >> x >> y;
    string s;
    cin >> s;
    for (int i = 0; i < n; i++) {
        cin >> iarr[i];
    }
    vector<int> roots;
    {
        int ind = 0;
        vector<int> st;
        for (auto& a : s) {
            if (a == '(') {
                if (!sz(st)) {
                    roots.push_back(ind);
                } else {
                    graph[st.back()].push_back(ind);
                }
                st.push_back(ind++);
            } else {
                st.pop_back();
            }
        }
    }
    for (auto& a : roots) {
        dfs(a, 0);
    }
    if (!x && y) {
        solve01();
    } else if (x && !y) {
        solve10();
    } else if (x && y) {
        solve11();
    } else {
        cout << 0 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}