#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

int encrypt(int arr[2]) {
	if(arr[0] > arr[1]) {
		return (arr[1] + 1) << 1;
	}
	return ((arr[0] + 1) << 1) | 1;
}

void solve() {
	int n, m;
	cin >> n >> m;
	string sh, sv;
	int hori[n], vert[m], costh[n], costv[m];
	{
		cin >> sh;
		for(int i = 0; i < n; i++) {
			hori[i] = sh[i] == 'E';
		}
	}
	{
		cin >> sv;
		for(int i = 0; i < m; i++) {
			vert[i] = sv[i] == 'S';
		}
	}
	for(int i = 0; i < n; i++) {
		cin >> costh[i];
	}
	for(int i = 0; i < m; i++) {
		cin >> costv[i];
	}
	int q;
	cin >> q;
	vector<array<int, 4>> queries[m];
	while(q--) {
		int x1, y1, x2, y2;
		cin >> x1 >> y1 >> x2 >> y2;
		x1--;
		y1--;
		x2--;
		y2--;
		if(x1 != x2 || y1 != y2) {
			queries[y1].push_back({x1, y1, x2, y2});
			if(y1 != y2) {
				queries[y2].push_back({x1, y1, x2, y2});
			}
		}
	}
	int ans = 1e9;
	string ah, av;
	for(int bni = 0; bni < (1 << n); bni++) {
		bitset<10> bn(bni);
		int dp[m + 1][(m + 1) * 2];
		pair<int, int> nxt[m + 1][(m + 1) * 2];
		memset(dp, -1, sizeof(dp));
		fill(dp[m], dp[m] + (m + 1) * 2, 0);
		for(int ind = m - 1; ind >= 0; ind--) {
			for(int ltype = 0; ltype < 2; ltype++) {
				for(int lind = 0; lind < max(1, ind); lind++) {
					int &cur = dp[ind][(lind << 1) | ltype];
					cur = 1e9;
					int last[2];
					last[ltype] = ind - 1;
					last[ltype ^ 1] = lind - 1;
					if(ind == 0) {
						last[0] = last[1] = -1;
					}
					bool poss[2];
					poss[0] = poss[1] = true;
					for(auto &[x1, y1, x2, y2]: queries[ind]) {
						bool left = y1 > y2;
						bool down = x1 < x2;
						// force this street to go to same direction
						auto force = [&]() -> void {
							poss[!down] = false;
						};
						if(x1 == x2) {
							if(!(bn[x1] ^ left)) {
								goto loop;
							}
							continue;
						}else if(y1 == y2) {
							force();
							continue;
						}
						if(bn[x1] ^ left) {
							if(bn[x2] ^ left) {
								// both going in my direction, just needs one vertical street
								if(last[down] < min(y1, y2) && ind == max(y1, y2)) {
									force();
								}
							}else {
								// destination street wrong, destination vertical must be right
								if(ind == y2) {
									force();
								}
							}
						}else {
							if(bn[x2] ^ left) {
								// source street wrong, source vertical must be right
								if(ind == y1) {
									force();
								}
							}else {
								// both streets wrong, both verticals must be right with one hori street in between
								for(int i = min(x1, x2) + 1; i < max(x1, x2); i++) {
									if(bn[i] ^ left) {
										goto end;
									}
								}
								goto loop;
								end:;
								force();
							}
						}
					}
					for(int i = 0; i < 2; i++) {
						if(poss[i]) {
							int nlast[2] = {last[0], last[1]};
							nlast[i] = ind;
							int cost = 0;
							if(i != vert[ind]) {
								cost = costv[ind];
							}
							int nenc = encrypt(nlast);
							if(cost + dp[ind + 1][nenc] < cur) {
								cur = cost + dp[ind + 1][nenc];
								nxt[ind][(lind << 1) | ltype] = {i, nenc};
							}
						}
					}
					loop:;
					dbg(ind, lind, ltype, cur);
				}
			}
		}
		int cost = dp[0][0];
		for(int i = 0; i < n; i++) {
			if(bn[i] ^ hori[i]) {
				cost += costh[i];
			}
		}
		if(cost < ans) {
			ans = cost;
			ah = av = "";
			for(int i = 0; i < n; i++) {
				ah += "WE"[bn[i]];
			}
			int j = 0;
			for(int i = 0; i < m; i++) {
				auto &[cur, next] = nxt[i][j];
				av += "NS"[cur];
				j = next;
			}
		}
	}
	if(ans >= 1e9) {
		cout << "impossible" << endl;
	}else {
		int cost = 0;
		for(int i = 0; i < n; i++) {
			cost += (ah[i] != sh[i]) * costh[i];
		}
		for(int i = 0; i < m; i++) {
			cost += (av[i] != sv[i]) * costv[i];
		}
		assert(ans == cost);
		cout << "possible" << endl;
		cout << ans << endl;
		cout << ah << endl;
		cout << av << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}