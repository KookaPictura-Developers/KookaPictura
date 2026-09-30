#pragma once

#include <memory>

#include <QtCore/QByteArray>
#include <QtCore/QTemporaryDir>

#include "frame.h"

namespace pictura::test {

// Redirects XDG_STATE_HOME to a throwaway dir for the guard's lifetime so a
// PicturaMainWindow never reads or writes the developer's real session state.
// Construct it before the window so it outlives the window's session save.
class ScopedStateHome {
public:
    ScopedStateHome()
        : valid_(dir_.isValid())
        , hadPrior_(qEnvironmentVariableIsSet("XDG_STATE_HOME"))
        , prior_(hadPrior_ ? qgetenv("XDG_STATE_HOME") : QByteArray())
    {
        if (valid_) {
            qputenv("XDG_STATE_HOME", dir_.path().toUtf8());
        }
    }

    ~ScopedStateHome()
    {
        if (!valid_) {
            return;
        }
        if (hadPrior_) {
            qputenv("XDG_STATE_HOME", prior_);
        } else {
            qunsetenv("XDG_STATE_HOME");
        }
    }

    ScopedStateHome(const ScopedStateHome&) = delete;
    ScopedStateHome& operator=(const ScopedStateHome&) = delete;

    bool isValid() const { return valid_; }

private:
    QTemporaryDir dir_;
    bool valid_;
    bool hadPrior_;
    QByteArray prior_;
};

inline std::unique_ptr<PicturaMainWindow> makeMainWindow(QWidget* parent = nullptr)
{
    return std::make_unique<PicturaMainWindow>(parent);
}

} // namespace pictura::test
