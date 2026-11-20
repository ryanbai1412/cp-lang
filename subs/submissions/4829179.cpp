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
    int n1, m;
    cin >> m >> n1;
    int iarr[n1];
    for (auto& a : iarr) {
        cin >> a;
    }
    vector<pair<int, long>> arr;
    {
        int ind = 0;
        auto nxt = [&]() -> int {
            return iarr[ind++];
        };
        auto push = [&](int x, long cnt) -> void {
            if (sz(arr) && arr.back().first == x) {
                arr.back().second += cnt;
            } else {
                arr.emplace_back(x, cnt);
            }
        };
        int rep = 0;
        while (ind < n1) {
            int x = nxt();
            if (x != rep) {
                push(x, 1);
            } else {
                x = nxt();
                int cnt = nxt();
                if (x == rep) {
                    push(rep, cnt + 1);
                } else {
                    if (!cnt) {
                        rep = x;
                    } else {
                        push(x, cnt + 3);
                    }
                }
            }
        }
    }
    long cadd = 3;
    map<int, long> spec;
    spec[0] = -3;
    auto cdiv = [&](long a, long b) -> long {
        return (a + b - 1) / b;
    };
    int crep = 0, mrep = 0;
    vector<int> ans;
    for (auto& [x, cnt] : arr) {
        long cx = (cnt / (m + 2)) * 3 + min(cnt % (m + 2), long(3));
        cadd += cx;
        long cspec = spec[x] += cdiv(cnt, m) * 3 - cx;
        if (cspec > 0) {
            spec.erase(x);
        } else if (cspec < -3) {
            assert(false);
            vector<int> td;
            for (auto& [k, v] : spec) {
                if (cadd + cspec + 3 < cadd + v) {
                    td.push_back(k);
                } else {
                    v = cadd + v - (cadd + cspec + 3);
                }
            }
            for (auto& a : td) {
                spec.erase(a);
            }
            cadd = cadd + cspec + 3;
            spec[x] = -3;
        }
    }
    {
        long cans = 0;
        for (auto& [_, v] : spec) {
            cans = min(cans, v);
        }
        dbg(cadd + cans);
    }
    cout << sz(ans) << endl;
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}