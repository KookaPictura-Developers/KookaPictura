#pragma once

#include <QtCore/QString>
#include <QtWidgets/QWidget>

namespace pictura {

// A CS6-style empty-state panel content widget hosted by a PanelGroup, used
// for panels whose contents are not implemented yet.
class PlaceholderPanel : public QWidget {
    Q_OBJECT

public:
    explicit PlaceholderPanel(const QString& title, const QString& message = QString(),
                              QWidget* parent = nullptr);
};

} // namespace pictura
