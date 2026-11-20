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

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[m][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
        x--;
        y--;
    }
    vector<pair<int, int>> ans;
    auto move = [&](int x, int y) -> void {
        ans.emplace_back(x, y);
        int top = -1;
        for (int i: {1, 0}) {
            if (arr[x][i] != -1) {
                swap(arr[x][i], top);
                break;
            }
        }
        assert(top != -1);
        for (int i: {0, 1}) {
            if (arr[y][i] == -1) {
                swap(arr[y][i], top);
                break;
            }
        }
        assert(top == -1);
    };
    while (true) {
        bool good = true;
        for (auto& [x, y] : arr) {
            good &= x == y;
        }
        if (good) {
            break;
        }
        vector<int> bot[n], top[n];
        for (int i = 0; i < m; i++) {
            auto& [b, t] = arr[i];
            if (t != -1) {
                top[t].push_back(i);
            } else if (b != -1) {
                top[b].push_back(i);
                bot[b].push_back(i);
            }
        }
        bool changed = false;
        for (int i = 0; i < n; i++) {
            for (auto& b : bot[i]) {
                for (auto& t : top[i]) {
                    if (b != t) {
                        move(t, b);
                        changed = true;
                        goto loop;
                    }
                }
            }
            loop:;
        }
        if (changed) {
            continue;
        }
        int open = -1;
        for (int i = 0; i < m; i++) {
            if (arr[i][0] == -1) {
                open = i;
                break;
            }
        }
        if (open == -1) {
            cout << -1 << endl;
            return;
        }
        for (auto& a : top) {
            if (sz(a) == 2) {
                move(a[0], open);
                changed = true;
                break;
            }
        }
        if (changed) {
            continue;
        }
        for (int i = 0; i < m; i++) {
            if (arr[i][1] != -1) {
                move(i, open);
                changed = true;
                break;
            }
        }
        assert(changed);
    }
    cout << sz(ans) << endl;
    for (auto& [x, y] : ans) {
        cout << x + 1 << " " << y + 1 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}