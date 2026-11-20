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

struct Node {
    deque<int> arr;
    map<int, int> freq;

    int size() const {
        return sz(arr);
    }

    void push(int x) {
        arr.push_back(x);
        freq[x]++;
    }

    void pop() {
        freq[arr.back()]--;
        arr.pop_back();
    }

    int cnt_freq(int x) {
        auto it = freq.find(x);
        return it == freq.end() ? 0 : it->second;
    }

    void upd_freq(const Node& n) {
        for (auto& [x, y] : n.freq) {
            freq[x] += y;
        }
    }

    void push_front(const Node& n) {
        auto& narr = n.arr;
        for (int i = sz(narr) - 1; i >= 0; i--) {
            arr.push_front(narr[i]);
        }
        upd_freq(n);
    }

    void push_back(const Node& n) {
        for (auto& a : n.arr) {
            arr.push_back(a);
        }
        upd_freq(n);
    }
};

void solve() {
    mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());
    auto rint = [&](long l, long r) -> long {
        return cowng() % (r - l + 1) + l;
    };
    int n, q;
    cin >> n >> q;
    Node arr[n];
    map<int, int> id;
    for (int i = 0; i < n; i++) {
        id[i] = i;
        int m;
        cin >> m;
        for (int j = 0; j < m; j++) {
            int x;
            cin >> x;
            arr[i].push(x);
        }
    }
    while (q--) {
        int t;
        cin >> t;
        if (t == 1) {
            int x, y;
            cin >> x >> y;
            x = id[x - 1];
            arr[x].push(y);
        } else if (t == 2) {
            int x;
            cin >> x;
            x = id[x - 1];
            arr[x].pop();
        } else if (t == 3) {
            int m;
            cin >> m;
            int carr[m];
            for (auto& a : carr) {
                cin >> a;
                a = id[a - 1];
            }
            long siz[m + 1];
            siz[0] = 0;
            for (int i = 0; i < m; i++) {
                siz[i + 1] = siz[i] + sz(arr[carr[i]]);
            }
            for (int i = 0; i < 40; i++) {
                long cv = rint(0, siz[m] - 1);
                int bind = int(upper_bound(siz, siz + m, cv) - siz) - 1;
                int cx = arr[carr[bind]].arr[cv - siz[bind]];
                long occur = 0;
                for (auto& a : carr) {
                    occur += arr[a].cnt_freq(cx);
                }
                if (occur > siz[m] / 2) {
                    cout << cx << endl;
                    goto loop;
                }
            }
            cout << -1 << endl;
        loop:;
        } else if (t == 4) {
            int a, b, c;
            cin >> a >> b >> c;
            a = id[a - 1];
            b = id[b - 1];
            c--;
            if (sz(arr[a]) < sz(arr[b])) {
                arr[b].push_front(arr[a]);
                id[c] = b;
            } else {
                arr[a].push_back(arr[b]);
                id[c] = a;
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}