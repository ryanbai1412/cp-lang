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

using ld = long double;
using cpx = complex<ld>;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

bool oleft(ld x1, ld y1, ld x2, ld y2, ld x3, ld y3) {
	cpx p1(x1, y1), p2(x2, y2), p3(x3, y3);
	p2 -= p1;
	p3 -= p1;
	return (p2 * (-p3)).imag() >= 0;
}

bool isect1(ld x1, ld y1, ld x2, ld y2, ld x3, ld y3, ld x4, ld y4) {
	return oleft(x1, y1, x2, y2, x3, y3) != oleft(x1, y1, x2, y2, x4, y4);
}

bool isect(ld x1, ld y1, ld x2, ld y2, ld x3, ld y3, ld x4, ld y4) {
	return isect1(x1, y1, x2, y2, x3, y3, x4, y4) && isect1(x3, y3, x4, y4, x1, y1, x2, y2);
}

void solve() {
	int n;
	cin >> n;
	int arr[n][2];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	dbg(isect(0, 0, 0, 1, -1, -1, 1, -1));
	int ans = 0;
	for (int i = 0; i < n; i++) {
		for (int j = i + 1; j < n; j++) {
			for (int a = 0; a < n; a++) {
				if (a != i && a != j) {
					for (int b = a + 1; b < n; b++) {
						if (isect(arr[i][0], arr[i][1], arr[j][0], arr[j][1], arr[a][0], arr[a][1],
								  arr[b][0], arr[b][1])) {
							dbg(i, j, a, b);
							goto loop;
						}
					}
				}
			}
			dbg(i, j);
			ans++;
		loop:;
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}