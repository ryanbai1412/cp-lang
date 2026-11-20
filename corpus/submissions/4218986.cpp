#pragma GCC optimize("Ofast,unroll-loops")

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

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
	input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
	input_pos = 0;

	if (input_len == 0)
		input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
	if (input_pos >= input_len)
		_update_input_buffer();

	return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
	bool negative = false;
	number = 0;

	while (!isdigit(next_char(false)))
		if (next_char() == '-')
			negative = true;

	do {
		number = 10 * number + (next_char() - '0');
	} while (isdigit(next_char(false)));

	if (negative)
		number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
	read_int(number);
	read_int(args...);
}

void _flush_output() {
	fwrite(output_buffer, sizeof(char), output_pos, stdout);
	output_pos = 0;
}

inline void write_char(char c) {
	if (output_pos == BUFFER_SIZE)
		_flush_output();

	output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
	if (number < 0) {
		write_char('-');
		number = -number;
	}

	int length = 0;

	while (number >= 10) {
		uint8_t lookup_value = lookup[number % 100];
		number /= 100;
		number_buffer[length++] = char((lookup_value & 15) + '0');
		number_buffer[length++] = char((lookup_value >> 4) + '0');
	}

	if (number != 0 || length == 0)
		write_char(char(number + '0'));

	for (int i = length - 1; i >= 0; i--)
		write_char(number_buffer[i]);

	if (after)
		write_char(after);
}

void init() {
	// Make sure _flush_output() is called at the end of the program.
	bool exit_success = atexit(_flush_output) == 0;
	assert(exit_success);

	for (int i = 0; i < 100; i++)
		lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

const int maxn = 2e6;

pair<int, int> cache[maxn + 1];

bool prime[maxn + 1];
vector<int> primes;

void pcomp() {
	memset(prime, 1, sizeof(prime));
	prime[0] = prime[1] = false;
	for (int i = 2; i <= maxn; i++) {
		if (prime[i]) {
			primes.push_back(i);
			if (long(i) * i <= maxn) {
				for (int j = i * i; j <= maxn; j += i) {
					prime[j] = false;
				}
			}
		}
	}
}

void solve() {
	int n;
	IO::read_int(n);
	if (n == 1) {
		IO::write_int(1, '\n');
	} else if (n == 2 || n == 3) {
		IO::write_char('-');
		IO::write_char('1');
		IO::write_char('\n');
	} else if (n == 4) {
		IO::write_int(3, ' ');
		IO::write_int(1, ' ');
		IO::write_int(4, ' ');
		IO::write_int(2, '\n');
	} else {
		int vis[n] {};
		int ct = 1;
		auto check = [&](int ind) -> bool {
			return 0 <= ind && ind < n && vis[ind] != ct;
		};
		int ind = int(upper_bound(begin(primes), end(primes), n) - primes.begin());
		if (cache[n].first) {
			auto [a, b] = cache[n];
			int cur = -1;
			for (int i = 0; i < n; i++) {
				if (!i) {
					cur = 0;
				} else {
					if (check(cur + a)) {
						cur += a;
					} else if (check(cur - b)) {
						cur -= b;
					}
				}
				IO::write_int(cur + 1, " \n"[i == n - 1]);
				vis[cur] = ct;
			}
			return;
		}
		auto process = [&](int limitl, int limitr) -> bool {
			for (int ai = 0; ai < 100 && ai < ind; ai++) {
				int a = primes[ai];
				for (int bi = ind - limitl; bi >= 0 && bi > ind - limitr; bi--) {
					int b = primes[bi];
					ct++;
					{
						int cur = -1;
						for (int i = 0; i < n; i++) {
							if (!i) {
								cur = 0;
							} else {
								if (check(cur + a)) {
									cur += a;
								} else if (check(cur - b)) {
									cur -= b;
								} else {
									goto loop;
								}
							}
							vis[cur] = ct;
						}
					}
					ct++;
					{
						cache[n] = {a, b};
						int cur = -1;
						for (int i = 0; i < n; i++) {
							if (!i) {
								cur = 0;
							} else {
								if (check(cur + a)) {
									cur += a;
								} else if (check(cur - b)) {
									cur -= b;
								}
							}
							IO::write_int(cur + 1, " \n"[i == n - 1]);
							vis[cur] = ct;
						}
					}
					return true;
				loop:;
				}
			}
			return false;
		};
		bool b = process(0, 10) || process(10, 25) || process(25, 40) || process(40, 60) || process(60, 100);
		assert(b);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	IO::init();
	pcomp();
	int t = 1;
	IO::read_int(t);
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}