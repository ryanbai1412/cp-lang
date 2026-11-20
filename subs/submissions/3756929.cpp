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

const int dx[4] {-1, 1, 0, 0}, dy[4] {0, 0, -1, 1};

struct Base {
	int n, m;
	vector<string> arr;
	vector<array<int, 4>> ans;
	int copt, clast;
	bool running;
	mt19937 cowng;
	string inputFile, outputFile;

	Base(): running(false), cowng(chrono::steady_clock::now().time_since_epoch().count()) {}

	void read() {
		cin >> n >> m;
		arr.resize(n);
		for(int i = 0; i < n; i++) {
			cin >> arr[i];
		}
	}

	void answer() {
		cout << sz(ans) << endl;
		for(auto &a: ans) {
			for(auto &b: a) {
				cout << b + 1 << " ";
			}
			cout << endl;
		}
	}

	bool ibs(int x, int y) {
		return 0 <= x && x < n && 0 <= y && y < m;
	}

	bool irc(int x, int y) {
		return ibs(x, y) && arr[x][y] == 'O';
	};

	bool iba(int x, int y) {
		return ibs(x, y) && arr[x][y] == '#';
	}

	void scale(int &x, int &y, int dir, int factor = 1) {
		x += dx[dir] * factor;
		y += dy[dir] * factor;
	}

	virtual void solve() = 0;
};

struct Solver5: Base {
	void solve() {
		read();
		for(int i = 0; i < n; i++) {
			for(int j = 0; j < m; j++) {
				int cnt = 0;
				for(int k = 0; k < 4; k++) {
					int ii = i, jj = j;
					scale(ii, jj, k);
					cnt += irc(ii, jj);
				}
				if(cnt == 4) {
					for(int k = 0; k < 4; k++) {
						int x = i, y = j;
						while(irc(x, y)) {
							scale(x, y, k);
						}
						for(int l = 0; l < 4; l++) {
							int xx = x, yy = y;
							scale(xx, yy, l);
							if(iba(xx, yy)) {
								for(int a = 0; iba(x, y); a++) {
									xx = x;
									yy = y;
									for(int b = 0; b < a; b++) {
										int px = xx, py = yy;
										scale(xx, yy, k ^ 1);
										ans.push_back({px, py, xx, yy});
									}
									scale(x, y, l);
								}
								break;
							}
						}
					}
				}
			}
		}
		answer();
	}
};

struct Solver8: Base {
	void solve() override {
		read();
		for(int i = 0; i < n; i++) {
			for(int j = 0; j < m; j++) {
				if(arr[i][j] == '#') {
					for(int k = 1;; k++) {
						for(int dir = 0; dir < 4; dir++) {
							int nx = i, ny = j;
							scale(nx, ny, dir, k);
							if(irc(nx, ny)) {
								int x = i, y = j;
								while(true) {
									int ncx = x, ncy = y;
									scale(ncx, ncy, dir);
									if(tie(ncx, ncy) == tie(nx, ny)) {
										break;
									}
									ans.push_back({x, y, ncx, ncy});
									tie(x, y) = tie(ncx, ncy);
								}
								goto loop;
							}
						}
					}
					loop:;
				}
			}
		}
		answer();
	}
};

void solve() {
	int t;
	cin >> t;
	Base *solver = nullptr;
	if(t == 5) {
		solver = new Solver5();
	}else if(t == 8) {
		solver = new Solver8();
	}
	if(solver) {
		solver->solve();
		delete solver;
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