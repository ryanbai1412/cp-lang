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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
istream& operator>>(istream& in, vector<T>& arr) {
    for (auto& a : arr) {
        in >> a;
    }
    return in;
}

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

vector<int> nxt(vector<int> arr) {
    int n = sz(arr);
    vector<int> l(arr.begin(), arr.begin() + n / 2), r(arr.begin() + n / 2, arr.end());
    reverse(begin(l), end(l));
    reverse(begin(r), end(r));
    vector<int> ans;
    while (sz(l) && sz(r)) {
        if (l.back() < r.back()) {
            ans.push_back(l.back());
            l.pop_back();
        } else {
            ans.push_back(r.back());
            r.pop_back();
        }
    }
    ans.insert(ans.end(), l.rbegin(), l.rend());
    ans.insert(ans.end(), r.rbegin(), r.rend());
    return ans;
}

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    cin >> arr;
    vector<int> vals[n + 1];
    for (int i = 0; i <= n; i++) {
        vals[i] = arr;
        arr = nxt(arr);
    }
    while (q--) {
        int t, ind;
        cin >> t >> ind;
        ind--;
        cout << vals[min(t, n)][ind] << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}