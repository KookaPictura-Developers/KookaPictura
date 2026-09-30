#pragma once

// Opt-in paint-path timing for the Qt frontend, enabled by the same
// `PICTURA_PAINT_TIMING=1` env var the Rust side reads. A no-op otherwise.
// Header-only so it can be included wherever a phase needs instrumenting.

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <locale>
#include <sstream>

namespace pictura {

inline int paintTimingLevel()
{
    static const int level = [] {
        const char* value = std::getenv("PICTURA_PAINT_TIMING");
        return value ? std::atoi(value) : 0;
    }();
    return level;
}

/// Logs its elapsed wall time under `label` on destruction.
class ScopedTimer {
public:
    explicit ScopedTimer(const char* label)
        : label_(label)
        , start_(std::chrono::steady_clock::now())
    {
    }

    ~ScopedTimer()
    {
        if (paintTimingLevel() <= 0) {
            return;
        }
        const double ms =
            std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start_)
                .count();
        // Classic locale so the decimal separator is always '.', regardless of
        // the user's locale.
        std::ostringstream line;
        line.imbue(std::locale::classic());
        line.setf(std::ios::fixed);
        line.precision(2);
        line << ms;
        std::fprintf(stderr, "[paint-timing]   %-30s %10s ms\n", label_, line.str().c_str());
    }

    ScopedTimer(const ScopedTimer&) = delete;
    ScopedTimer& operator=(const ScopedTimer&) = delete;

private:
    const char* label_;
    std::chrono::steady_clock::time_point start_;
};

} // namespace pictura
