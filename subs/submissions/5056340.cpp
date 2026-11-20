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

constexpr int maxn = 105;

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

pair<int, int>& operator^=(pair<int, int>& a, const pair<int, int>& b) {
    a.first ^= b.first;
    a.second ^= b.second;
    return a;
}

int k;
vector<pair<int, int>> v[maxn];

pair<int, int> get(int a, int b) {
    if (b < sz(v[a])) {
        return v[a][b];
    }
    return {a, b - sz(v[a])};
}

void pcomp() {
    for (int i = 1; i < maxn; i++) {
        set<pair<int, int>> missing;
        for (int j = 0; j < i; j++) {
            int cind = k * (i - j);
            for (int a = 0; a < cind - sz(v[j]); a++) {
                missing.emplace(j, a);
            }
        }
        for (int j = 0; j < i; j++) {
            int cind = k * (i - j);
            for (int a = cind; a < sz(v[j]); a++) {
                missing.erase(v[j][a]);
            }
        }
        v[i] = {begin(missing), end(missing)};
    }
}

void solve() {
    int n;
    cin >> n;
    int arr[n][2];
    for (auto& [a, b] : arr) {
        cin >> a >> b;
    }
    pair<int, int> ans {};
    for (auto& [a, b] : arr) {
        ans ^= get(a, b);
    }
    if (ans.first || ans.second) {
        cout << "Win" << endl;
    } else {
        cout << "Loss" << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> k >> tcs;
    pcomp();
    for (int i = 0; i < 5; i++) {
        for (int j = 0; j < 5; j++) {
            cerr << get(i, j) << " \n"[j == 4];
        }
    }
    while (tcs--) {
        solve();
    }
}