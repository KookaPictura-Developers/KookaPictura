#include <QtTest/QtTest>

#include <QtCore/QStringList>
#include <QtGui/QIcon>
#include <QtGui/QPixmap>

#include "icons.h"

class IconAssetsTest : public QObject {
    Q_OBJECT

private slots:
    void panelGlyphsResolve();
    void pathGlyphsResolve();
    void infoGlyphsResolve();
    void unknownIdIsNull();
};

void IconAssetsTest::panelGlyphsResolve()
{
    const QStringList ids = {
        QStringLiteral("layers.search"),
        QStringLiteral("layers.kindShape"),
        QStringLiteral("layers.kindSmartObject"),
        QStringLiteral("layers.reset"),
    };
    for (const QString& id : ids) {
        const QIcon icon = pictura::icon(id);
        QVERIFY2(!icon.isNull(), qPrintable(id + QStringLiteral(" resolves")));
        QVERIFY2(!icon.pixmap(16, 16).isNull(),
                 qPrintable(id + QStringLiteral(" renders at 16px")));
    }
}

void IconAssetsTest::pathGlyphsResolve()
{
    const QStringList ids = {
        QStringLiteral("path.thumbnail"),
        QStringLiteral("path.fill"),
        QStringLiteral("path.stroke"),
        QStringLiteral("path.loadSelection"),
        QStringLiteral("path.makeWorkPath"),
        QStringLiteral("path.newPath"),
        QStringLiteral("path.delete"),
    };
    for (const QString& id : ids) {
        const QIcon icon = pictura::icon(id);
        QVERIFY2(!icon.isNull(), qPrintable(id + QStringLiteral(" resolves")));
        QVERIFY2(!icon.pixmap(16, 16).isNull(),
                 qPrintable(id + QStringLiteral(" renders at 16px")));
    }
}

void IconAssetsTest::infoGlyphsResolve()
{
    const QStringList ids = {
        QStringLiteral("info.crosshair"),
        QStringLiteral("info.bounds"),
        QStringLiteral("info.protractor"),
    };
    for (const QString& id : ids) {
        const QIcon icon = pictura::icon(id);
        QVERIFY2(!icon.isNull(), qPrintable(id + QStringLiteral(" resolves")));
        QVERIFY2(!icon.pixmap(16, 16).isNull(),
                 qPrintable(id + QStringLiteral(" renders at 16px")));
    }
}

void IconAssetsTest::unknownIdIsNull()
{
    QVERIFY(pictura::icon(QStringLiteral("no.such.icon")).isNull());
}

QTEST_MAIN(IconAssetsTest)
#include "tst_icon_assets.moc"
