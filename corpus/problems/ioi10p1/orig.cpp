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

int Theory(int M, int L, int W) {
    printf("%d %d %d\n", M, L, W);
    fflush(stdout);
    int r;
    if (scanf("%d", &r) != 1) exit(1);
    return r;
}
void Solve() {
    int arr[3] {1, 1, 1};
    while (true) {
        int x = Theory(arr[0], arr[1], arr[2]);
        if (!x) {
            return;
        }
        arr[x - 1]++;
    }
}
int main() {
    int t;
    if (scanf("%d", &t) != 1) return 1;
    while (t--) Solve();
}
