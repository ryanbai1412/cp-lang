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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

namespace ExitCode {
const int AC = 0, WA = 1, PE = 2, IE = 3, PARTIAL = 7;
}

namespace Input {
struct PresentationError : public std::exception {};
struct IntegerParsingError : public PresentationError {};
struct FloatingPointParsingError : public PresentationError {};
struct WhitespaceError : public PresentationError {};
struct RequirementFailure : public std::exception {};
struct IntegerRangeError : public RequirementFailure {};
struct FloatingPointRangeError : public RequirementFailure {};
struct RegexError : public RequirementFailure {};

struct ValidatorErrorHandler {
   public:
    template <class E>
    void handle(const E& e) {
        throw e;
    }
};

struct CheckerErrorHandler {
   public:
    void handle(const PresentationError&) {
        exit(ExitCode::PE);
    }
    void handle(const RequirementFailure&) {
        exit(ExitCode::WA);
    }
};

const bool IDENTICAL = true, STANDARD = false;

template <class ErrorHandler, const bool IDENTICAL_WHITESPACE>
struct Reader : public ErrorHandler {
   private:
    std::unique_ptr<std::ifstream> streamPtr;
    std::istream& stream;
    bool hasLast;
    bool skipToNextLine;
    char last;

    bool isWhitespace(char c) {
        if (IDENTICAL_WHITESPACE)
            return c == ' ' || c == '\n';
        return isspace(c);
    }

    bool isNewLine(char c) {
        if (IDENTICAL_WHITESPACE)
            return c == '\n';
        return c == '\n' || c == '\r';
    }

    bool isEOF(char c) {
        return c == std::char_traits<char>::eof();
    }

    char peekChar() {
        if (!hasLast)
            last = stream.get();
        hasLast = true;
        return last;
    }

    char getChar() {
        char ret = peekChar();
        hasLast = false;
        return ret;
    }

    void skipWhitespace() {
        while (isWhitespace(peekChar()) &&
               (skipToNextLine || !isNewLine(peekChar())))
            getChar();
        skipToNextLine = false;
    }

   public:
    template <class E>
    void require(bool expr, const E& e) {
        if (!expr)
            ErrorHandler::handle(e);
    }

    template <class... Args>
    Reader(std::istream& stream, Args&&... args)
        : ErrorHandler(std::forward<Args>(args)...),
          stream(stream),
          hasLast(false),
          skipToNextLine(true) {}

    template <class... Args>
    Reader(const std::string& fileName, Args&&... args)
        : ErrorHandler(std::forward<Args>(args)...),
          streamPtr(std::make_unique<std::ifstream>(fileName)),
          stream(*streamPtr),
          hasLast(false),
          skipToNextLine(true) {}

    int64_t readInt(
        int64_t minValid = std::numeric_limits<int64_t>::min(),
        int64_t maxValid = std::numeric_limits<int64_t>::max()) {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        std::string token = "";
        while (isdigit(peekChar()) || peekChar() == '-')
            token.push_back(getChar());
        int64_t ret = 0;
        try {
            ret = std::stoll(token);
        } catch (std::invalid_argument&) {
            ErrorHandler::handle(IntegerParsingError());
        } catch (std::out_of_range&) {
            ErrorHandler::handle(IntegerParsingError());
        }
        require(minValid <= ret && ret <= maxValid, IntegerRangeError());
        return ret;
    }

    ld readFloat(
        ld minValid = std::numeric_limits<ld>::lowest(),
        ld maxValid = std::numeric_limits<ld>::max()) {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        std::string token = "";
        while (isdigit(peekChar()) || peekChar() == '-' || peekChar() == '.')
            token.push_back(getChar());
        ld ret = 0;
        try {
            ret = std::stold(token);
        } catch (std::invalid_argument&) {
            ErrorHandler::handle(FloatingPointParsingError());
        } catch (std::out_of_range&) {
            ErrorHandler::handle(FloatingPointParsingError());
        }
        require(minValid <= ret && ret <= maxValid, FloatingPointRangeError());
        return ret;
    }

    std::string readString(std::regex rgx) {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        std::string ret = "";
        while (!isWhitespace(peekChar()) && !isEOF(peekChar()))
            ret.push_back(getChar());
        require(std::regex_match(ret, rgx), RegexError());
        return ret;
    }

    std::string readString() {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        std::string ret = "";
        while (!isWhitespace(peekChar()) && !isEOF(peekChar()))
            ret.push_back(getChar());
        return ret;
    }

    char readChar(std::regex rgx) {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        char ret = getChar();
        require(std::regex_match(std::string(1, ret), rgx), RegexError());
        return ret;
    }

    char readChar() {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        return getChar();
    }

    std::string readLine(std::regex rgx) {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        std::string ret = "";
        while (!isNewLine(peekChar()) && !isEOF(peekChar()))
            ret.push_back(getChar());
        require(std::regex_match(ret, rgx), RegexError());
        return ret;
    }

    std::string readLine() {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        std::string ret = "";
        while (!isNewLine(peekChar()) && !isEOF(peekChar()))
            ret.push_back(getChar());
        return ret;
    }

    void readSpace() {
        require(isWhitespace(peekChar()) && !isNewLine(peekChar()),
                WhitespaceError());
        getChar();
    }

    void readNewLine() {
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        skipToNextLine = true;
        require((!IDENTICAL_WHITESPACE && isEOF(peekChar())) ||
                    isNewLine(getChar()),
                WhitespaceError());
    }

    bool atEOF() {
        skipToNextLine = true;
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        return isEOF(peekChar());
    }

    void readEOF() {
        skipToNextLine = true;
        if (!IDENTICAL_WHITESPACE)
            skipWhitespace();
        require(isEOF(getChar()), WhitespaceError());
    }
};

using Validator = Reader<ValidatorErrorHandler, IDENTICAL>;
using StandardReader = Reader<CheckerErrorHandler, STANDARD>;
using IdenticalReader = Reader<CheckerErrorHandler, IDENTICAL>;
}  // namespace Input

namespace sol {
template <int MOD>
struct Mint {
    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }
    static constexpr int inv(int x) {
        int y = MOD, vx = 1, vy = 0;
        while (x) {
            int k = y / x;
            y %= x;
            vy -= k * vx;
            swap(x, y);
            swap(vx, vy);
        }
        assert(y == 1);
        if (vy < 0) {
            vy += MOD;
        }
        return vy;
    }
    
    int x;

    constexpr Mint() : x(0) {}
    constexpr Mint(long x) : x(norm(x)) {}

    static Mint from_unsafe(int x) {
        Mint m;
        m.x = x;
        return m;
    }

    friend istream& operator>>(istream& in, Mint& m) {
        long x;
        in >> x;
        m = x;
        return in;
    }
    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    explicit operator int() const {
        return x;
    }
    explicit operator long() const {
        return x;
    }

    bool operator==(const Mint& m) const {
        return x == m.x;
    }
    bool operator!=(const Mint& m) const {
        return x != m.x;
    }

    Mint operator-() const {
        return !x ? 0 : Mint::from_unsafe(MOD - x);
    }
    Mint operator+() const {
        return *this;
    }

    Mint& operator++() {
        x++;
        if (x == MOD) {
            x = 0;
        }
        return *this;
    }
    Mint& operator--() {
        if (!x) {
            x = MOD - 1;
        } else {
            x--;
        }
        return *this;
    }

    Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    Mint& operator-=(const Mint& m) {
        x -= m.x;
        if (x < 0) {
            x += MOD;
        }
        return *this;
    }

    Mint& operator*=(const Mint& m) {
        x = int((long(x) * m.x) % MOD);
        return *this;
    }
    Mint& operator/=(const Mint& m) {
        return *this *= m.inv();
    }
    Mint inv() const {
        return Mint::from_unsafe(Mint::inv(x));
    }

    friend Mint operator++(Mint a, int) {
        return ++a;
    }
    friend Mint operator--(Mint a, int) {
        return --a;
    }
    friend Mint operator+(Mint a, const Mint& b) {
        return a += b;
    }
    friend Mint operator-(Mint a, const Mint& b) {
        return a -= b;
    }
    friend Mint operator*(Mint a, const Mint& b) {
        return a *= b;
    }
    friend Mint operator/(Mint a, const Mint& b) {
        return a /= b;
    }
};

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

using mint = Mint<998244353>;

constexpr int dx[4] = {-1, 1, 0, 0}, dy[4] = {0, 0, -1, 1};

int solve(vector<vector<int>> arr) {
    for (auto& a : arr) {
        for (auto& b : a) {
            b--;
        }
    }
    int n = sz(arr);
    pair<int, int> loc[n * n];
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            loc[arr[i][j]] = {i, j};
        }
    }
    mint dp[n][n];
    bool vis[n][n] {};
    auto ibs = [&](int x, int y) -> bool {
        return 0 <= x && x < n && 0 <= y && y < n;
    };
    for (auto& [x, y] : loc) {
        vis[x][y] = true;
        bool adj = false;
        for (int i = 0; i < 4; i++) {
            int cx = x + dx[i], cy = y + dy[i];
            if (ibs(cx, cy) && vis[cx][cy]) {
                adj = true;
                dp[x][y] += dp[cx][cy];
            }
        }
        if (!adj) {
            dp[x][y] = 1;
        }
    }
    mint ans = 0;
    for (auto& a : dp) {
        ans += accumulate(a, a + n, mint(0));
    }
    return int(ans);
}
}  // namespace sol

void solve() {
    Input::Validator validator(cin);
    int n = validator.readInt(1, 1000);
    validator.readNewLine();
    vector<vector<int>> arr(n, vector<int> (n));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            arr[i][j] = validator.readInt(1, n * n);
            if (j < n - 1) {
                validator.readSpace();
            } else {
                validator.readNewLine();
            }
        }
    }
    bool vis[n * n] {};
    for (auto& a : arr) {
        for (auto& b : a) {
            vis[b - 1] = true;
        }
    }
    for (auto& a : vis) {
        assert(a);
    }
    cout << sol::solve(arr) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}