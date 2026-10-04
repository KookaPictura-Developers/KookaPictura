// The Character, Paragraph, and Glyphs panels (#65): they edit the Type tools'
// shared options, and a glyph lands at the open type edit's caret.

#include <QtTest/QtTest>

#include "commands.h"
#include "panels/glyphs_panel.h"
#include "panels/panel_column.h"
#include "selftest_paint_fixture.h"

#include "pictura_app/src/cxxqt_object/type_tools.cxxqt.h"

#include <QtGui/QAction>
#include <QtGui/QFontDatabase>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFontComboBox>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QToolButton>

#include "qt_test_support.h"

namespace {

using paint_fixture::Fixture;

QImage seed()
{
    QImage image(160, 120, QImage::Format_RGB32);
    image.fill(Qt::white);
    return image;
}

QString typeLayer(const Fixture& f)
{
    for (int i = 0; i < f.view->layer_count(); ++i) {
        if (f.view->layer_is_type(QString::number(i))) {
            return QString::number(i);
        }
    }
    return QString();
}

} // namespace

class TypePanelsTest : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void characterPanel();
    void paragraphPanel();
    void glyphsPanel();

private:
    void showFrame();
    // Show `panel` through its Window > Panels command.
    bool openPanel(const char* command, const QString& panel);
    template <typename T>
    T* child(const QString& name) const
    {
        return window_->findChild<T*>(name);
    }

    pictura::test::ScopedStateHome stateHome_;
    std::unique_ptr<pictura::PicturaMainWindow> window_;
};

void TypePanelsTest::initTestCase()
{
    QVERIFY(stateHome_.isValid());
    window_ = pictura::test::makeMainWindow();
    QVERIFY(window_ != nullptr);
}

void TypePanelsTest::showFrame()
{
    window_->show();
    QVERIFY(QTest::qWaitForWindowExposed(window_.get()));
    QCoreApplication::processEvents();
}

bool TypePanelsTest::openPanel(const char* command, const QString& panel)
{
    QAction* action = window_->registry()->action(QString::fromLatin1(command));
    if (!action || !action->isCheckable()) {
        return false;
    }
    window_->registry()->refresh();
    if (!action->isChecked()) {
        action->trigger();
    }
    QCoreApplication::processEvents();
    return window_->panelColumn()->isPanelVisible(panel);
}

void TypePanelsTest::characterPanel()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_character_panel"));
    QVERIFY2(f.ok(), "fixture");
    showFrame();
    const pictura::TypeOptions saved = f.tools->typeOptions();

    // Type > Panels > Character opens it; the Window entry then reads checked.
    QAction* typeCharacter = frame.registry()->action(pictura::commandIdForPath(
        {QStringLiteral("Type"), QStringLiteral("Panels"), QStringLiteral("Character")}));
    QVERIFY(typeCharacter);
    frame.registry()->refresh();
    QVERIFY(typeCharacter->isEnabled());
    typeCharacter->trigger();
    QVERIFY(frame.panelColumn()->isPanelVisible(QStringLiteral("characterPanel")));
    QVERIFY(openPanel(pictura::command_ids::WindowPanelsCharacter,
                      QStringLiteral("characterPanel")));

    auto* size = child<QComboBox>(QStringLiteral("characterSize"));
    auto* antialias = child<QComboBox>(QStringLiteral("characterAntialias"));
    auto* family = child<QFontComboBox>(QStringLiteral("characterFamily"));
    QVERIFY(size && antialias && family);

    // The panel edits the shared options, as the options bar does.
    size->setEditText(QStringLiteral("30"));
    emit size->lineEdit()->editingFinished();
    QCOMPARE(f.tools->typeOptions().size, 30.0);
    size->setEditText(QStringLiteral("5000"));
    emit size->lineEdit()->editingFinished();
    QCOMPARE(f.tools->typeOptions().size, 30.0);
    QCOMPARE(size->currentText(), QStringLiteral("30"));
    antialias->setCurrentIndex(0);
    QVERIFY(!f.tools->typeOptions().antialias);
    const QStringList families = QFontDatabase::families();
    QVERIFY(!families.isEmpty());
    const QString other = families.first() == saved.family ? families.last() : families.first();
    family->setCurrentFont(QFont(other));
    QCOMPARE(f.tools->typeOptions().family, family->currentFont().family());

    // And follows them when they change elsewhere.
    pictura::TypeOptions o = f.tools->typeOptions();
    o.size = 12.0;
    o.antialias = true;
    f.tools->setTypeOptions(o);
    QCOMPARE(size->currentText(), QStringLiteral("12"));
    QCOMPARE(antialias->currentIndex(), 1);

    // What the type model lacks is shown but disabled.
    for (QSpinBox* spin : frame.findChild<QWidget*>(QStringLiteral("characterPanel"))
                              ->findChildren<QSpinBox*>()) {
        QVERIFY(!spin->isEnabled());
    }

    // The Type options bar's panel button toggles the Character panel.
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    QToolButton* toggle = nullptr;
    for (auto* button : frame.findChildren<QToolButton*>(
             QStringLiteral("optionsToggleCharacterPanel"))) {
        toggle = button->isVisible() ? button : toggle;
    }
    QVERIFY(toggle && !toggle->icon().isNull());
    toggle->click();
    QVERIFY(!frame.panelColumn()->isPanelVisible(QStringLiteral("characterPanel")));
    toggle->click();
    QVERIFY(frame.panelColumn()->isPanelVisible(QStringLiteral("characterPanel")));
    f.tools->setTypeOptions(saved);
}

void TypePanelsTest::paragraphPanel()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_paragraph_panel"));
    QVERIFY2(f.ok(), "fixture");
    showFrame();
    const pictura::TypeOptions saved = f.tools->typeOptions();
    QVERIFY(openPanel(pictura::command_ids::WindowPanelsParagraph,
                      QStringLiteral("paragraphPanel")));
    auto* left = child<QToolButton>(QStringLiteral("paragraphAlignLeft"));
    auto* center = child<QToolButton>(QStringLiteral("paragraphAlignCenter"));
    auto* right = child<QToolButton>(QStringLiteral("paragraphAlignRight"));
    QVERIFY(left && center && right);

    center->click();
    QCOMPARE(f.tools->typeOptions().justification, 2);
    pictura::TypeOptions o = f.tools->typeOptions();
    o.justification = 1;
    f.tools->setTypeOptions(o);
    QVERIFY(right->isChecked());

    // A vertical Type tool turns the buttons to top / centre / bottom.
    frame.setActiveTool(pictura::ToolId::VerticalType);
    QCOMPARE(left->toolTip(), QStringLiteral("Top align text"));
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    QCOMPARE(left->toolTip(), QStringLiteral("Left align text"));
    for (QDoubleSpinBox* field : frame.findChild<QWidget*>(QStringLiteral("paragraphPanel"))
                                     ->findChildren<QDoubleSpinBox*>()) {
        QVERIFY(!field->isEnabled());
    }
    f.tools->setTypeOptions(saved);
}

void TypePanelsTest::glyphsPanel()
{
    pictura::PicturaMainWindow& frame = *window_;
    Fixture f(frame, seed(), QStringLiteral("pictura_glyphs_panel"));
    QVERIFY2(f.ok(), "fixture");
    showFrame();
    QVERIFY(openPanel(pictura::command_ids::WindowPanelsGlyphs, QStringLiteral("glyphsPanel")));
    auto* panel = frame.findChild<pictura::GlyphsPanel*>();
    auto* grid = child<QListWidget>(QStringLiteral("glyphsGrid"));
    auto* subset = child<QComboBox>(QStringLiteral("glyphsSubset"));
    auto* hint = child<QLabel>(QStringLiteral("glyphsHint"));
    QVERIFY(panel && grid && subset && hint);

    // Basic Latin is at most its 95 printable characters; the entire font is more.
    subset->setCurrentIndex(1);
    const int basic = grid->count();
    QVERIFY2(basic > 50 && basic <= 95, "Basic Latin");
    subset->setCurrentIndex(0);
    QVERIFY(grid->count() > basic);
    subset->setCurrentIndex(1);
    const auto items = grid->findItems(QStringLiteral("Z"), Qt::MatchExactly);
    QCOMPARE(items.size(), 1);

    // Without an open type edit there is nowhere for the glyph to go.
    frame.setActiveTool(pictura::ToolId::HorizontalType);
    QVERIFY(!f.tools->textActive());
    panel->chooseGlyph(items.first());
    QVERIFY(hint->isVisible());

    // With one, it lands at the caret.
    f.drag({QPointF(20, 60)});
    QVERIFY(f.tools->textActive());
    QTest::keyClicks(f.canvas, QStringLiteral("a"));
    panel->chooseGlyph(items.first());
    QVERIFY(!hint->isVisible());
    QVERIFY(f.tools->textActive());
    const int base = f.view->history_index();
    QVERIFY(f.tools->commitText());
    QVERIFY2(f.committedOnce(base, "Horizontal Type"), "commit");
    QCOMPARE(QString(pictura::type_layer_text(*f.view, typeLayer(f))), QStringLiteral("aZ"));
}

QTEST_MAIN(TypePanelsTest)
#include "tst_type_panels.moc"
