#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("ssse3,sse4.2,sse4a,bmi,bmi2,avx2")

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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 4105, maxb = maxn / 64 + 1;
const long mod = 1e9 + 7;
const string chars = "ACGT";

struct BS {
	u64 arr[maxb];

	BS() : arr() {}

	void set(int ind) {
		arr[ind >> 6] |= u64(1) << (ind & 63);
	}

	int operator^(const BS& y) const {
		int ans = 0;
		for (int i = 0; i < maxb; i++) {
			ans += __builtin_popcountll(arr[i] ^ y.arr[i]);
		}
		return ans;
	}
};

BS arrb[maxn][4];

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	string arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	auto hsh = [&](int ind) -> long {
		long h1 = 0, h2 = 0;
		for (int i = 0; i < m; i++) {
			h1 = (h1 * 7 + int(chars.find(arr[ind][i])) + 1) % mod;
			h2 = (h2 * 11 + int(chars.find(arr[ind][i])) + 1) % mod;
		}
		return h1 * mod + h2;
	};
	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	int ord[n], conv[n], cind = 0;
	iota(ord, ord + n, 0);
	shuffle(ord, ord + n, cowng);
	bool bad[n] {}, checked[n][n] {};
	map<long, int> vis;
	bool ac = true;
	for (auto& a : ord) {
		auto [it, inserted] = vis.emplace(hsh(a), cind);
		if (!inserted) {
			bad[it->second] = true;
			continue;
		}
		for (int c = 0; c < 4; c++) {
			for (int j = 0; j < m; j++) {
				if (arr[a][j] == chars[c]) {
					ac &= c < 2;
					arrb[cind][c].set(j);
				}
			}
		}
		conv[cind] = a;
		cind++;
	}
	k *= 2;
	for (int i = 0; i < cind; i++) {
		if (bad[i]) {
			continue;
		}
		for (int j = 0; j < cind; j++) {
			if (i == j || checked[i][j]) {
				continue;
			}
			int cur = 0;
			if (ac) {
				for (int c = 0; c < 2; c++) {
					cur += arrb[i][c] ^ arrb[j][c];
				}
			} else {
				for (int c = 0; c < 4; c++) {
					cur += arrb[i][c] ^ arrb[j][c];
				}
			}
			checked[i][j] = checked[j][i] = true;
			if (cur != k) {
				bad[i] = bad[j] = true;
				goto loop;
			}
		}
		cout << conv[i] + 1 << endl;
		return;
	loop:;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}