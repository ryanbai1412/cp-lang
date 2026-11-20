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

void solve() {
    Input::Validator validator(cin);
    int n = validator.readInt(1, 3000);
    validator.readNewLine();
    int arr[n][n];
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
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}