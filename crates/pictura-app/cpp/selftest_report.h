#pragma once
#include <cstdarg>
namespace pictura {
class SelfTestReport {
public:
    void begin(const char* name);              // set current check; emit SUITE when suite changes
    void pass(const char* fmt = nullptr, ...); // emit PASS <suite> <name> [detail]; count passed
    void skip(const char* reason);             // emit SKIP <suite> <name> <reason>; count skipped
    int  fail(int code, const char* fmt, ...); // emit FAIL <suite> <name> <code> <message>; count failed; return code
    int  finish();                             // emit SUMMARY passed=.. failed=.. skipped=..; return failed?1:0
private:
    const char* suiteFor(const char* name) const; // "m47_float_close"->"m47"; bare->"core"
    void emit(const char* keyword, const char* fmt, va_list ap);
    const char* suite_ = "core";
    const char* lastSuite_ = nullptr;
    char name_[160] = "unnamed";
    int passed_ = 0, failed_ = 0, skipped_ = 0;
};
SelfTestReport& selfTest();
}  // namespace pictura
#define ST_BEGIN(name) pictura::selfTest().begin(name)
#define ST_PASS(...) pictura::selfTest().pass(__VA_ARGS__)
#define ST_SKIP(reason) pictura::selfTest().skip(reason)
#define ST_FAIL(code, ...) return pictura::selfTest().fail((code), __VA_ARGS__)
#define ST_FINISH() return pictura::selfTest().finish()
