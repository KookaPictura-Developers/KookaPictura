#pragma once

#include <QtCore/QString>
#include <QtWidgets/QDockWidget>

namespace pictura {

// A dockable panel with a CS6-style empty state, used for panels whose
// contents are not implemented yet.
class PlaceholderPanel : public QDockWidget {
    Q_OBJECT

public:
    explicit PlaceholderPanel(const QString& title, const QString& message = QString(),
                              QWidget* parent = nullptr);
};

} // namespace pictura
