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
#define sz(x) int((x).size())

const long mod = 1e9 + 7;

void upd(long k, long &a, long &b) {
	if (a < b) {
		long sub = min(k, b - a);
		k -= sub;
		a += sub;
	} else {
		long sub = min(k, a - b);
		k -= sub;
		b += sub;
	}
	a += k / 2;
	b += (k + 1) / 2;
}

void solve() {
	int n, k;
	cin >> n >> k;
	long a[n], b[n];
	for (auto& x : a) {
		cin >> x;
	}
	for (auto& x : b) {
		cin >> x;
	}
	sort(a, a + n);
	sort(b, b + n);
	upd(k, a[n - 1], b[n - 1]);
	long ans = 0;
	for (int i = 0; i < n; i++) {
		ans = (ans + a[i] * b[i]) % mod;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}