#ifdef LOCAL
#include "pcomp_headers.h"

#else
#include "bits/stdc++.h"
#include "ext/pb_ds/assoc_container.hpp"
#include "ext/pb_ds/detail/standard_policies.hpp"
#endif

using namespace std;
using namespace __gnu_pbds;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int(x.size())

#ifdef LOCAL
const int maxn = 10, bsize = 2;
#else
const int maxn = 2e5, bsize = 500;
#endif
const int m = maxn / bsize + 1;

int n, q, cind;
int arr[maxn], mdiv[maxn + 1], v[m][maxn + 1];
vector<pair<int, int>> pf;
vector<int> factors[maxn + 1];

void gen(int ind, int cur) {
	if(ind == sz(pf)) {
		factors[cind].push_back(cur);
	}else {
		gen(ind + 1, cur);
		for(int i = 0; i < pf[ind].second; i++) {
			gen(ind + 1, cur *= pf[ind].first);
		}
	}
}

void init() {
	mdiv[0] = mdiv[1] = 1;
	for(int i = 2; i <= maxn; i++) {
		if(mdiv[i] == 0) {
			mdiv[i] = i;
			if(maxn / i >= i) {
				for(int j = i * i; j <= maxn; j += i) {
					if(mdiv[j] == 0) {
						mdiv[j] = i;
					}
				}
			}
		}
	}
	for(cind = 1; cind <= maxn; cind++) {
		pf.clear();
		int j = cind;
		while(j > 1) {
			int cur = mdiv[j], cnt = 0;
			while(mdiv[j] == cur) {
				cnt++;
				j /= cur;
			}
			pf.emplace_back(cur, cnt);
		}
		gen(0, 1);
	}
}

void build() {
	int bind = 0;
	for(int i = 0; i < maxn; i += bsize, bind++) {
		int r = min(maxn, i + bsize);
		for(int j = i; j < r; j++) {
			for(auto &a: factors[arr[j]]) {
				v[bind][a]++;
			}
		}
	}
}

void update(int ind, int x) {
	int bind = ind / bsize;
	for(auto &a: factors[arr[ind]]) {
		v[bind][a]--;
	}
	for(auto &a: factors[x]) {
		v[bind][a]++;
	}
	arr[ind] = x;
}

int query(int l, int r, int x) {
	int lind = l / bsize, rind = r / bsize;
	if(lind == rind) {
		int ans = 0;
		for(int i = l; i <= r; i++) {
			ans += arr[i] % x == 0;
		}
		return ans;
	}
	int ans = 0;
	for(int i = l; i < bsize * (lind + 1); i++) {
		ans += arr[i] % x == 0;
	}
	for(int i = rind * bsize; i <= r; i++) {
		ans += arr[i] % x == 0;
	}
	for(int i = lind + 1; i < rind; i++) {
		ans += v[i][x];
	}
	return ans;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	init();
	cin >> n >> q;
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	build();
	while(q--) {
		int type;
		cin >> type;
		if(type == 1) {
			int l, r, x;
			cin >> l >> r >> x;
			l--;
			r--;
			cout << query(l, r, x) << endl;
		}else {
			int ind, x;
			cin >> ind >> x;
			ind--;
			update(ind, x);
		}
	}
}