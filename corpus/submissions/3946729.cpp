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
#ifdef DEBUG
	mt19937_64 cowng(1412);
#else
	mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());
#endif
	auto rint = [&](long l, long r) -> long {
		return cowng() % (r - l + 1) + l;
	};
	auto read = [&](int& x) -> void {
		cin >> x;
		if (x == -1) {
			while (true)
				;
		}
	};
	int queries = 0;
	auto write = [&](long x) -> void {
		if (++queries == 32000) {
			exit(0);
		}
		assert(0 <= x && x < (long(1) << 60));
		cout << x << endl;
	};

	int n;
	cin >> n;
	for (int i = 0; i < n; i++) {
		write(0);
		int bits;
		read(bits);
		int tog = bits - 1;
		long cur = long(1) << tog;
		while (true) {
			long cguess = rint(0, (long(1) << tog) - 1);
			cguess |= cur;
			write(cguess);
			int missed;
			read(missed);
			assert(missed % 2 == 0);
			missed /= 2;
			if (!missed) {
				break;
			}
			missed--;
			tog = missed;
			cur |= (cguess >> missed) << missed;
			cur ^= long(1) << missed;
		}
	}
}

int main() {
	cin.sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
//	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}