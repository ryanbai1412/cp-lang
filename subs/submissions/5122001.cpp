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

int a, b, c;

void empty(int x) {
    cout << "EMPTY " << x + 1 << endl;
}

void place(int x) {
    cout << "PLACE " << x + 1 << endl;
}

void discard() {
    cout << "DISCARD" << endl;
}

int read() {
    char c;
    cin >> c;
    return string("RGB").find(c);
}

bool solve1() {
    if (a + b < 46) {
        return false;
    }
    for (int i = 0; i < 100; i++) {
        int x = read();
        if (x < 2) {
            place(x);
        } else {
            discard();
        }
    }
    return true;
}

bool solve2() {
    if (b + c - 2 * a < 46) {
        return false;
    }
    int cnt[3] {}, id[3], cid = 0;
    memset(id, -1, sizeof(id));
    for (int i = 0; i < 100; i++) {
        int x = read();
        if (++cnt[x] > a) {
            int& xid = id[x];
            if (xid == -1) {
                xid = cid++;
            }
            place(xid);
        } else {
            discard();
        }
    }
    return true;
}

bool solve3() {
    if (a + c - b < 46) {
        return false;
    }
    int cnt[3] {}, id[3] {0, 1, -1};
    for (int i = 0; i < 100; i++) {
        int x = read();
        if (++cnt[x] > b && id[x] == -1) {
            swap(id[x], id[0]);
            empty(id[x]);
        }
        if (id[x] == -1) {
            discard();
        } else {
            place(id[x]);
        }
    }
    return true;
}

void solve() {
    int arr[3];
    for (auto& a : arr) {
        cin >> a;
    }
    sort(begin(arr), end(arr));
    a = arr[0];
    b = arr[1];
    c = arr[2];
    solve1() || solve2() || solve3();
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}