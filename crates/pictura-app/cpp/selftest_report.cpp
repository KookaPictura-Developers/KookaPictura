#include "selftest_report.h"

#include <cstdio>

namespace pictura {

SelfTestReport& selfTest() {
    static SelfTestReport instance;
    return instance;
}

void SelfTestReport::emit(const char* keyword, const char* fmt, va_list ap) {
    std::fprintf(stderr, "pictura self-test: %s %s %s", keyword, suite_, name_);
    if (fmt && *fmt) {
        std::fputc(' ', stderr);
        std::vfprintf(stderr, fmt, ap);
    }
    std::fputc('\n', stderr);
    std::fflush(stderr);
}

void SelfTestReport::begin(const char* name) {
    if (name && *name) {
        std::snprintf(name_, sizeof(name_), "%s", name);
    } else {
        std::snprintf(name_, sizeof(name_), "unnamed");
    }
    const char* suite = suite_;
    if (!suiteEmitted_) {
        suiteEmitted_ = true;
        std::fprintf(stderr, "pictura self-test: SUITE %s %s\n", suite, name_);
        std::fflush(stderr);
    }
}

void SelfTestReport::pass(const char* fmt, ...) {
    ++passed_;
    va_list ap;
    va_start(ap, fmt);
    emit("PASS", fmt, ap);
    va_end(ap);
}

void SelfTestReport::skip(const char* reason) {
    ++skipped_;
    std::fprintf(stderr, "pictura self-test: SKIP %s %s", suite_, name_);
    if (reason && *reason) {
        std::fputc(' ', stderr);
        std::fputs(reason, stderr);
    }
    std::fputc('\n', stderr);
    std::fflush(stderr);
}

int SelfTestReport::fail(int code, const char* fmt, ...) {
    ++failed_;
    std::fprintf(stderr, "pictura self-test: FAIL %s %s %d", suite_, name_, code);
    if (fmt && *fmt) {
        std::fputc(' ', stderr);
        va_list ap;
        va_start(ap, fmt);
        std::vfprintf(stderr, fmt, ap);
        va_end(ap);
    }
    std::fputc('\n', stderr);
    std::fflush(stderr);
    return code;
}

int SelfTestReport::finish() {
    std::fprintf(stderr, "pictura self-test: SUMMARY passed=%d failed=%d skipped=%d\n",
                 passed_, failed_, skipped_);
    std::fflush(stderr);
    return failed_ ? 1 : 0;
}

}  // namespace pictura
