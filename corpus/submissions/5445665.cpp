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

int g_result(int a, int b) {
    if (a > b) {
        return -g_result(b, a);
    }
    if (a == b) {
        return 0;
    }
    for (auto& [ca, cb] : initializer_list<pair<int, int>> {
             {0, 2}, {0, 3}, {1, 3}, {2, 4}, {3, 4}}) {
        if (a == ca && b == cb) {
            return 1;
        }
    }
    return -1;
}

void solve() {
    int n, na, nb;
    cin >> n >> na >> nb;
    auto read = [&](int m) -> vector<int> {
        int arr[m];
        for (auto& a : arr) {
            cin >> a;
        }
        vector<int> ans(n);
        for (int i = 0; i < n; i++) {
            ans[i] = arr[i % m];
        }
        return ans;
    };
    auto arra = read(na), arrb = read(nb);
    int cnt[2] {};
    for (int i = 0; i < n; i++) {
        int cr = g_result(arra[i], arrb[i]);
        if (cr == -1) {
            cnt[1]++;
        } else if (cr == 1){
            cnt[0]++;
        }
    }
    cout << cnt[0] << " " << cnt[1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}