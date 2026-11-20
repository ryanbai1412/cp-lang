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

using V = vector<tuple<int, int, bool>>;

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

long check(int n, int m, const V &arr) {
	map<int, bool> mp;
	for (auto& [x, y, b] : arr) {
		int cur = (y ^ b) & 1;
		auto [it, inserted] = mp.emplace(x, cur);
		if (it->second != cur) {
			return 0;
		}
	}
	return bpow(2, n - sz(mp));
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	V arr(k);
	for (auto& [x, y, b] : arr) {
		char c;
		cin >> c >> x >> y;
		x--;
		y--;
		b = c == '+';
	}
	long hori = check(n, m, arr);
	for (auto& [x, y, b] : arr) {
		swap(x, y);
	}
	long vert = check(m, n, arr);
	long ans = hori + vert;
	if (hori && vert) {
		if (k) {
			ans--;
		} else {
			ans -= 2;
		}
	}
	ans %= mod;
	if (ans < 0) {
		ans += mod;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}