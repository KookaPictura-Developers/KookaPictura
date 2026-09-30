#include <QtTest/QtTest>

#include "frame.h"
#include "panels/channels_panel.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "qt_test_support.h"

class ChannelsPanelTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanup();
    void rowsAndThumbnails();
    void togglingIsLocalOnly();
    void refreshKeepsRows();
    void grayscaleDocumentShowsSingleGrayRow();
    void thumbnailsTrackDocument();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
    pictura::PictureView* view_ = nullptr;
    pictura::ChannelsPanel* panel_ = nullptr;
};

void ChannelsPanelTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void ChannelsPanelTest::cleanup()
{
    if (window_) {
        while (window_->activeDocumentIndex() >= 0) {
            window_->closeDocument(window_->activeDocumentIndex(), false);
        }
    }
    view_ = nullptr;
    panel_ = nullptr;
}

void ChannelsPanelTest::rowsAndThumbnails()
{
    const bool created = window_->newDocument(QStringLiteral("Channels"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::ChannelsPanel*>(QStringLiteral("channelsPanel"));
    QVERIFY2(created && view_ && panel_, "channels fixture");

    panel_->setView(view_);
    panel_->refresh();
    QCOMPARE(panel_->channelCountForTest(), 4);
    QCOMPARE(panel_->channelNameForTest(0), QStringLiteral("RGB"));
    QCOMPARE(panel_->channelNameForTest(1), QStringLiteral("Red"));
    QCOMPARE(panel_->channelNameForTest(2), QStringLiteral("Green"));
    QCOMPARE(panel_->channelNameForTest(3), QStringLiteral("Blue"));
    for (int i = 0; i < panel_->channelCountForTest(); ++i) {
        QVERIFY2(!panel_->channelThumbnailForTest(i).isNull(), "thumbnail pixmap");
    }
}

void ChannelsPanelTest::togglingIsLocalOnly()
{
    const bool created = window_->newDocument(QStringLiteral("Toggle"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::ChannelsPanel*>(QStringLiteral("channelsPanel"));
    QVERIFY2(created && view_ && panel_, "toggle fixture");
    panel_->setView(view_);
    panel_->refresh();

    QVERIFY2(panel_->channelVisibleForTest(1), "red starts visible");
    panel_->toggleChannelForTest(1);
    QVERIFY2(!panel_->channelVisibleForTest(1), "red toggled off");
    QVERIFY2(panel_->channelVisibleForTest(0) && panel_->channelVisibleForTest(2)
                 && panel_->channelVisibleForTest(3),
             "only red flipped");
}

void ChannelsPanelTest::refreshKeepsRows()
{
    const bool created = window_->newDocument(QStringLiteral("Stable"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::ChannelsPanel*>(QStringLiteral("channelsPanel"));
    QVERIFY2(created && view_ && panel_, "stable fixture");
    panel_->setView(view_);
    panel_->refresh();
    const int before = panel_->channelCountForTest();
    panel_->refresh();
    QCOMPARE(panel_->channelCountForTest(), before);
}

void ChannelsPanelTest::grayscaleDocumentShowsSingleGrayRow()
{
    const bool created = window_->newDocument(QStringLiteral("Gray"), 16, 16,
                                              QStringLiteral("grayscale"), 8,
                                              QStringLiteral("white"));
    view_ = window_->activeView();
    panel_ = window_->findChild<pictura::ChannelsPanel*>(QStringLiteral("channelsPanel"));
    QVERIFY2(created && view_ && panel_, "grayscale fixture");
    panel_->setView(view_);
    QCOMPARE(panel_->channelCountForTest(), 1);
    QCOMPARE(panel_->channelNameForTest(0), QStringLiteral("Gray"));
}

void ChannelsPanelTest::thumbnailsTrackDocument()
{
    panel_ = window_->findChild<pictura::ChannelsPanel*>(QStringLiteral("channelsPanel"));
    QVERIFY(panel_ != nullptr);
    panel_->setView(nullptr);
    QVERIFY2(panel_->channelThumbnailForTest(0).isNull(), "no document -> no thumbnail");

    const bool created = window_->newDocument(QStringLiteral("Thumbs"), 16, 16,
                                              QStringLiteral("rgb"), 8, QStringLiteral("white"));
    view_ = window_->activeView();
    QVERIFY2(created && view_, "thumbs fixture");
    panel_->setView(view_);
    QVERIFY2(!panel_->channelThumbnailForTest(0).isNull(), "document -> thumbnail");
}

QTEST_MAIN(ChannelsPanelTest)
#include "tst_channels_panel.moc"
