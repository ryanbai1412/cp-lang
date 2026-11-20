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

using u64 = uint64_t;

u64 xorswap(u64 x) {
	x ^= x << 49;
	x ^= x >> 23;
	return x;
}

const u64 mod = (u64(1) << 59) - 1, base = 31;

int conv(char c) {
	if (c <= 'Z') {
		return c - 'A' + 1;
	}
	return c - 'a' + 1;
}

const int bsize = 1 << 20;

int buf_ind;
char buf[bsize];

const int hsize = 1 << 18, hmask = hsize - 1, hmx = 4;
const int vsize = 1 << 21, vmask = vsize - 1;

int hind[hsize], vis[vsize];
u64 hashes[hsize][hmx];

void solve() {
	fread_unlocked(buf, bsize, 1, stdin);
	auto read_int = [&]() -> int {
		int x = 0;
		char c;
		while ('0' <= (c = buf[buf_ind++])) {
			x = x * 10 + c - '0';
		}
		return x;
	};
	int n = read_int(), m = read_int();
	for (int i = 0; i < n; i++) {
		char c;
		u64 x = 0;
		while (' ' < (c = buf[buf_ind++])) {
			x = (x * base + conv(c)) & mod;
		}
		x = xorswap(x);
		u64 key = x & hmask;
		hashes[key][hind[key]++] = x;
	}
	for (int i = 1; i <= m; i++) {
		static int s[505];
		int sind = 0;
		char c;
		while (' ' < (c = buf[buf_ind++])) {
			s[sind++] = conv(c);
		}
		int ans = 0;
		for (int j = 0; j < sind; j++) {
			u64 chsh = 0;
			for (int k = 0; j + k < sind && k < 25; k++) {
				chsh = (chsh * base + s[j + k]) & mod;
				u64 x = xorswap(chsh);
				u64 key = x & hmask;
#pragma GCC unroll hmx
				for (int a = 0; a < hind[key]; a++) {
					if (hashes[key][a] == x) {
						u64 xkey = x & vmask;
						ans += vis[xkey] != i;
						vis[xkey] = i;
						break;
					}
				}
			}
		}
		cout << ans << endl;
	}
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