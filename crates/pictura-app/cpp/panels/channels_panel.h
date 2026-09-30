#pragma once

#include <QtCore/QList>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtGui/QPixmap>
#include <QtWidgets/QWidget>

class QLabel;
class QToolButton;
class QVBoxLayout;

namespace pictura {

class PictureView;

class ChannelsPanel : public QWidget {
    Q_OBJECT

public:
    explicit ChannelsPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    void refresh();

    int channelCountForTest() const;
    QString channelNameForTest(int index) const;
    QPixmap channelThumbnailForTest(int index) const;
    bool channelVisibleForTest(int index) const;
    void toggleChannelForTest(int index);

private:
    struct Row {
        QString name;
        QToolButton* eye = nullptr;
        QLabel* thumbnail = nullptr;
    };

    void rebuildRows(const QStringList& names);

    PictureView* view_ = nullptr;
    QVBoxLayout* layout_ = nullptr;
    QList<Row> rows_;
    QString mode_;
};

} // namespace pictura
