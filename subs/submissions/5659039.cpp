#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

vector<vector<int>> vals;

void dfs(int ind, int left) {
    static vector<int> carr;

    if (!ind) {
        if (!left) {
            vals.push_back(carr);
        }
        return;
    }
    for (int i = 0; i <= left; i++) {
        carr.push_back(i);
        dfs(ind - 1, left - i);
        carr.pop_back();
    }
}

void pcomp() {
    dfs(9, 11);
}

void solve1() {
    int n;
    cin >> n;
    bool vis[n] {};
    cout << 16 << endl;
    int one_sum = 0;
    for (int i = 0; i < n - 13; i++) {
        int ind;
        cin >> ind;
        vis[ind] = true;
        one_sum = (one_sum + ind) % n;
        cout << 1 << endl;
    }
    vector<int> not_one;
    for (int i = 0; i < n; i++) {
        if (!vis[i]) {
            not_one.push_back(i);
        }
    }
    auto cmask = vals[one_sum / 2];
    for (int i = 0; i < 10; i++) {
        for (int j = 0; j < cmask[i]; j++) {
            int ind;
            cin >> ind;
            vis[ind] = true;
            cout << 2 + i << endl;
        }
    }
    int ind;
    cin >> ind;
    not_one.erase(find(begin(not_one), end(not_one), ind));
    vis[ind] = true;
    int nvis = -1;
    for (int i = 0; i < n; i++) {
        if (!vis[i]) {
            assert(nvis == -1);
            nvis = i;
        }
    }
    assert(sz(not_one) == 12);
    cout << 11 + (find(begin(not_one), end(not_one), nvis) - not_one.begin()) / 2
         << endl;
}

void solve2() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    vector<int> ones;
    for (int i = 0; i < n; i++) {
        if (arr[i] == 1) {
            ones.push_back(i);
        }
    }

    if (sz(ones) != n - 13) {
        vector<int> cmask(9);
        for (auto& a : arr) {
            if (2 <= a && a <= 10) {
                cmask[a - 2]++;
            }
        }
        int os2 = int(find(begin(vals), end(vals), cmask) - vals.begin());
        vector<int> out;
        for (int one_sum : {os2 * 2, os2 * 2 + 1}) {
            for (auto& a : ones) {
                one_sum = (one_sum - a) % n;
            }
            one_sum = (-one_sum) % n;
            if (one_sum < 0) {
                one_sum += n;
            }
            out.push_back(one_sum);
        }
        cout << out[0] << " " <<out[1] << endl;
        return;
    }

    vector<int> flags;
    for (int i = 0; i < n; i++) {
        if (arr[i] >= 11) {
            flags.push_back(i);
        }
    }
    assert(1 <= sz(flags) && sz(flags) <= 2);
    if (sz(flags) == 2) {
        cout << flags[0] << " " << flags[1] << endl;
        return;
    }
    vector<int> not_one;
    for (int i = 0; i < n; i++) {
        if (2 <= arr[i] && arr[i] <= 10) {
            not_one.push_back(i);
        }
    }
    int nbad = (arr[flags[0]] - 11) * 2;
    cout << not_one[nbad] << " " << not_one[nbad + 1] << endl;
}

void solve() {
    int ty;
    cin >> ty;
    if (ty == 1) {
        solve1();
    } else {
        assert(ty == 2);
        solve2();
    }
}

int main() {
    // cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    pcomp();
    solve();
}