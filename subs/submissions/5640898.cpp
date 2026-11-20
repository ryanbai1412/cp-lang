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
#define sz(x) int((x).size())

struct FastCin {
    char buf[10 << 20], *i = buf;

    FastCin() {
        fread_unlocked(buf, 1, sizeof(buf), stdin);
    }

    FastCin& operator>>(int& x) {
        while (*i < '-') {
            i++;
        }

        bool neg = false;
        if (*i == '-') {
            neg = true;
            i++;
        }

        x = 0;

        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }

        if (neg) {
            x = -x;
        }
        return *this;
    }
} in;

constexpr int logn = 19, maxn = 3e5 + 5;

int lift[logn][maxn];

void solve() {
    int n;
    in >> n;

    int arr[n];
    for (auto& a : arr) {
        in >> a;
        a = -a;
    }

    auto level = [&](int ind) -> int { return max(0, arr[ind]); };

    for (auto& a : lift) {
        fill(a, a + n, -1);
    }

    // first thing in stack u < x
    auto query = [&](int u, int x) -> int {
        if (u == -1) {
            return -1;
        } else if (level(u) < x) {
            return u;
        }
        for (int i = logn - 1; i >= 0; i--) {
            int nu = lift[i][u];
            if (nu != -1 && level(nu) >= x) {
                u = nu;
            }
        }
        return lift[0][u];
    };

    auto do_lift = [&](int u) -> void {
        for (int i = 1; i < logn; i++) {
            if (lift[i - 1][u] == -1) {
                lift[i][u] = -1;
            } else {
                lift[i][u] = lift[i - 1][lift[i - 1][u]];
            }
        }
    };

    int cst = -1;

    for (int i = 0; i < n; i++) {
        if (arr[i] > 0) {
            int undo = query(cst, arr[i]);
            assert(undo != -1);

            lift[0][i] = query(undo - 1, arr[i]);
            do_lift(i);
        }

        cst = i;

        int ans = query(cst, 1);
        if (ans == -1) {
            cout << 0 << endl;
        } else {
            cout << -arr[ans] << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}