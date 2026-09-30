#include <QtTest/QtTest>

#include "frame.h"
#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include "qt_test_support.h"

class SmokeTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void frameOpensOffscreen();
    void stateHomeIsolationRestoresPriorValue();

private:
    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void SmokeTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void SmokeTest::frameOpensOffscreen()
{
    QCOMPARE(qApp->platformName(), QStringLiteral("offscreen"));
    QVERIFY(window_->newDocument(QStringLiteral("Untitled"), 512, 512, QStringLiteral("rgb"), 8,
                                 QStringLiteral("white")));
    QVERIFY(window_->activeView() != nullptr);
}

void SmokeTest::stateHomeIsolationRestoresPriorValue()
{
    qputenv("XDG_STATE_HOME", QByteArray("/prior-sentinel"));
    {
        pictura::test::ScopedStateHome guard;
        QVERIFY(guard.isValid());
        QVERIFY(qgetenv("XDG_STATE_HOME") != QByteArray("/prior-sentinel"));
    }
    QCOMPARE(qgetenv("XDG_STATE_HOME"), QByteArray("/prior-sentinel"));

    qunsetenv("XDG_STATE_HOME");
    {
        pictura::test::ScopedStateHome guard;
        QVERIFY(guard.isValid());
        QVERIFY(qEnvironmentVariableIsSet("XDG_STATE_HOME"));
    }
    QVERIFY(!qEnvironmentVariableIsSet("XDG_STATE_HOME"));
}

QTEST_MAIN(SmokeTest)
#include "tst_smoke.moc"
