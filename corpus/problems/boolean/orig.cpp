// Boolean: flip the final value once per `not`.
#include <iostream>
#include <string>

int main() {
    std::string w;
    bool flip = false, v = false;
    while (std::cin >> w) {
        if (w == "not") flip = !flip;
        else v = w == "True";
    }
    std::cout << ((v != flip) ? "True" : "False") << "\n";
}
