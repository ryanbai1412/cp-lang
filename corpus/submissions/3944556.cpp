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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n, m;
	scanf("%d%d", &n, &m);
	int a = n, b = n;
	for (int i = 1; i <= m; i++) {
		int u, v, x, y;
		scanf("%d%d%d%d", &u, &v, &x, &y);
		u *= v;
		x *= y;
		a -= u;
		b -= x;
		if (a <= 0) {
			if (b <= 0) {
				printf("It's a tie at round %d!\n", i);
				return;
			}
			printf("Team 1 wins at round %d!\n", i);
			return;
		} else if (b <= 0) {
			printf("Team 2 wins at round %d!\n", i);
			return;
		}
	}
	cout << "Oh no!" << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}