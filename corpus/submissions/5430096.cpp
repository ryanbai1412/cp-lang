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

struct S {
    mutable int k;
    mutable vector<pair<int, int>> v;

    bool operator<(const S& s) const {
        return k < s.k;
    }
};

void solve() {
    int n;
    cin >> n;
    pair<int, int> arr[n];
    for (auto& [a, b] : arr) {
        string s;
        cin >> s;
        int ind = int(s.find('-'));
        a = stoi(s.substr(0, ind));
        b = stoi(s.substr(ind + 1));
        if (a > b) {
            swap(a, b);
        }
    }
    sort(arr, arr + n, greater<>());
    set<S> mp;
    for (auto& [x, y] : arr) {
        dbg(x, y);
        auto it = mp.lower_bound({y, {}});
        if (it == mp.end()) {
            mp.insert({y, {{x, y}}});
            continue;
        }
        it->k = y;
        it->v.emplace_back(x, y);
    }
    cout << sz(mp) << endl;
    for (auto& [_k, v] : mp) {
        reverse(begin(v), end(v));
        for (int i = 0; i < sz(v); i++) {
            auto& [a, b] = v[i];
            cout << a << "-" << b << " \n"[i == sz(v) - 1];
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}