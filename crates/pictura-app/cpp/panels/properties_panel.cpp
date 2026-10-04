#include "properties_panel.h"

#include "color_picker_dialog.h"
#include "curve_widget.h"
#include "icons.h"
#include "layers_panel_internal.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/adjustment_edit.cxxqt.h"
#include "pictura_app/src/cxxqt_object/clipping.cxxqt.h"

#include <QtCore/QTimer>
#include <QtGui/QPainter>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QSlider>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

#include <algorithm>
#include <cmath>
#include <map>
#include <memory>

namespace pictura {

namespace {

// A pause this long after the last live edit (a wheel, arrow keys, or a curve
// drag, none of which has a release to hang the commit on) ends the gesture.
constexpr int kCommitDelayMs = 500;

enum Page { NothingPage, AdjustmentPage, LayerPage };

QIcon swatch(const QColor& color)
{
    QPixmap pixmap(16, 16);
    pixmap.fill(color);
    QPainter p(&pixmap);
    p.setPen(QColor(0, 0, 0, 160));
    p.drawRect(pixmap.rect().adjusted(0, 0, -1, -1));
    return QIcon(pixmap);
}

QToolButton* footerButton(QWidget* parent, const QString& iconId, const QString& tip,
                          const QString& name)
{
    auto* button = new QToolButton(parent);
    button->setObjectName(name);
    button->setIcon(icon(iconId));
    button->setToolTip(tip);
    button->setAutoRaise(true);
    return button;
}

} // namespace

PropertiesPanel::PropertiesPanel(QWidget* parent)
    : QWidget(parent)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(6, 6, 6, 6);
    header_ = new QLabel(this);
    header_->setObjectName(QStringLiteral("propertiesHeader"));
    QFont bold = header_->font();
    bold.setBold(true);
    header_->setFont(bold);
    layout->addWidget(header_);
    // One scroll area for every page, the footer outside it: the panel's
    // minimum size stays small, so it never crowds the panel column.
    auto* scroll = new QScrollArea(this);
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    scroll->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Ignored);
    stack_ = new QStackedWidget(scroll);
    scroll->setWidget(stack_);
    layout->addWidget(scroll, 1);

    auto* nothing = new QLabel(QStringLiteral("No Properties"), stack_);
    nothing->setAlignment(Qt::AlignCenter);
    nothing->setEnabled(false);
    stack_->addWidget(nothing);

    adjustmentPage_ = new QWidget(stack_);
    controls_ = new QVBoxLayout(adjustmentPage_);
    controls_->setContentsMargins(0, 0, 0, 0);
    controls_->addStretch(1);
    stack_->addWidget(adjustmentPage_);

    // CS6's footer: Clip to Layer, Reset, Toggle Visibility, Delete.
    footer_ = new QWidget(this);
    auto* footer = new QHBoxLayout(footer_);
    footer->setContentsMargins(0, 0, 0, 0);
    clip_ = footerButton(footer_, QStringLiteral("layers.clipMask"),
                         QStringLiteral("This adjustment clips to the layer (click to affect "
                                        "all layers below)"),
                         QStringLiteral("propertiesClip"));
    clip_->setCheckable(true);
    auto* reset = footerButton(footer_, QStringLiteral("edit.undo"),
                               QStringLiteral("Reset to adjustment defaults"),
                               QStringLiteral("propertiesReset"));
    visible_ = footerButton(footer_, QStringLiteral("layers.eyeOn"),
                            QStringLiteral("Toggle layer visibility"),
                            QStringLiteral("propertiesVisible"));
    auto* remove = footerButton(footer_, QStringLiteral("layers.delete"),
                                QStringLiteral("Delete this adjustment layer"),
                                QStringLiteral("propertiesDelete"));
    footer->addStretch(1);
    for (QToolButton* button : {clip_, reset, visible_, remove}) {
        footer->addWidget(button);
    }
    layout->addWidget(footer_);

    layerPage_ = new QWidget(stack_);
    info_ = new QFormLayout(layerPage_);
    stack_->addWidget(layerPage_);

    commitTimer_ = new QTimer(this);
    commitTimer_->setSingleShot(true);
    commitTimer_->setInterval(kCommitDelayMs);
    connect(commitTimer_, &QTimer::timeout, this, &PropertiesPanel::commit);

    connect(clip_, &QToolButton::clicked, this, [this]() {
        commit();
        if (view_) {
            clipping_toggle(*view_, path_);
        }
        refresh();
    });
    connect(reset, &QToolButton::clicked, this, [this]() {
        commit();
        // The controls stay; only their values reload.
        if (view_ && adjustment_reset(*view_, path_)) {
            dirty_ = true;
            commit();
            refresh();
        }
    });
    connect(visible_, &QToolButton::clicked, this, [this]() {
        commit();
        int row = -1;
        activePath(&row);
        if (view_ && row >= 0) {
            view_->set_layers_visible(QStringList{path_}, !view_->layer_row_visible(row));
        }
        refresh();
    });
    connect(remove, &QToolButton::clicked, this, [this]() {
        commit();
        if (view_) {
            view_->delete_layers(QStringList{path_});
        }
        refresh();
    });
    refresh();
}

void PropertiesPanel::setView(PictureView* view)
{
    if (view != view_) {
        commit();
        builtFor_.clear();
    }
    view_ = view;
    refresh();
}

QString PropertiesPanel::activePath(int* row) const
{
    *row = -1;
    if (!view_ || !view_->has_document()) {
        return QString();
    }
    const QString active = view_->active_layer_path();
    for (int i = 0; !active.isEmpty() && i < view_->layer_row_count(); ++i) {
        if (view_->layer_row_path(i) == active) {
            *row = i;
            return active;
        }
    }
    return QString();
}

void PropertiesPanel::refresh()
{
    int row = -1;
    const QString path = activePath(&row);
    if (path != path_) {
        commit();
        builtFor_.clear();
    }
    path_ = path;
    if (row < 0) {
        showNothing();
        return;
    }
    const QStringList page = adjustment_page(*view_, path_);
    if (page.isEmpty()) {
        showLayer(row);
        return;
    }
    clip_->setChecked(view_->layer_row_clipping(row));
    visible_->setIcon(icon(view_->layer_row_visible(row) ? QStringLiteral("layers.eyeOn")
                                                         : QStringLiteral("layers.eyeOff")));
    showAdjustment(page);
}

void PropertiesPanel::showNothing()
{
    header_->setText(QStringLiteral("No Properties"));
    stack_->setCurrentIndex(NothingPage);
    footer_->hide();
}

void PropertiesPanel::showLayer(int row)
{
    header_->setText(QStringLiteral("Layer Properties"));
    stack_->setCurrentIndex(LayerPage);
    footer_->hide();
    while (info_->rowCount() > 0) {
        info_->removeRow(0);
    }
    const QString kind = view_->layer_is_type(path_) ? QStringLiteral("type")
                                                     : view_->layer_row_kind(row);
    const std::map<QString, QString> kinds = {
        {QStringLiteral("pixel"), QStringLiteral("Pixel")},
        {QStringLiteral("type"), QStringLiteral("Type")},
        {QStringLiteral("group"), QStringLiteral("Group")},
        {QStringLiteral("background"), QStringLiteral("Background")},
        {QStringLiteral("adjustment"), QStringLiteral("Adjustment")}};
    const auto it = kinds.find(kind);
    info_->addRow(QStringLiteral("Kind:"),
                  new QLabel(it != kinds.end() ? it->second : kind, layerPage_));
    const QStringList rect = view_->layer_rect(path_).split(QLatin1Char(' '));
    if (rect.size() == 4 && kind != QStringLiteral("group")) {
        const int left = rect[0].toInt();
        const int top = rect[1].toInt();
        info_->addRow(QStringLiteral("Size:"),
                      new QLabel(QStringLiteral("%1 x %2 px")
                                     .arg(rect[2].toInt() - left)
                                     .arg(rect[3].toInt() - top),
                                 layerPage_));
        info_->addRow(QStringLiteral("Position:"),
                      new QLabel(QStringLiteral("X %1, Y %2").arg(left).arg(top), layerPage_));
    }
    QString blend = view_->layer_row_blend(row);
    for (const BlendEntry& entry : kBlends) {
        if (blend == QLatin1String(entry.key)) {
            blend = QLatin1String(entry.name);
        }
    }
    info_->addRow(QStringLiteral("Blend Mode:"), new QLabel(blend, layerPage_));
    const auto percent = [](int v) { return QStringLiteral("%1%").arg(qRound(v * 100.0 / 255.0)); };
    info_->addRow(QStringLiteral("Opacity:"),
                  new QLabel(percent(view_->layer_row_opacity(row)), layerPage_));
    info_->addRow(QStringLiteral("Fill:"), new QLabel(percent(view_->layer_row_fill(row)), layerPage_));
    info_->addRow(QStringLiteral("Mask:"),
                  new QLabel(view_->layer_row_has_mask(row) ? QStringLiteral("Layer Mask")
                                                            : QStringLiteral("None"),
                             layerPage_));
    const int lock = view_->layer_row_lock(row);
    QStringList locks;
    if (lock & 0x01) {
        locks << QStringLiteral("Transparency");
    }
    if (lock & 0x02) {
        locks << QStringLiteral("Pixels");
    }
    if (lock & 0x04) {
        locks << QStringLiteral("Position");
    }
    info_->addRow(QStringLiteral("Locks:"),
                  new QLabel(locks.isEmpty() ? QStringLiteral("None") : locks.join(QStringLiteral(", ")),
                             layerPage_));
}

void PropertiesPanel::showAdjustment(const QStringList& page)
{
    stack_->setCurrentIndex(AdjustmentPage);
    footer_->show();
    // Rebuild only for another layer or another set of controls; otherwise
    // reload values, except mid-gesture, when the controls lead.
    QString signature = path_;
    for (const QString& line : page) {
        signature += QLatin1Char('\n') + line.section(QLatin1Char('\t'), 0, 1);
    }
    if (signature != builtFor_) {
        buildAdjustment(page);
        builtFor_ = signature;
    } else if (!dirty_) {
        loadAdjustment(page);
    }
}

void PropertiesPanel::clearAdjustment()
{
    while (controls_->count() > 1) {
        QLayoutItem* item = controls_->takeAt(0);
        delete item->widget();
        delete item;
    }
    rows_.clear();
    groups_ = nullptr;
    note_ = nullptr;
    curve_ = nullptr;
    curveChannel_ = nullptr;
}

void PropertiesPanel::buildAdjustment(const QStringList& page)
{
    clearAdjustment();
    loading_ = true;
    for (const QString& line : page) {
        const QStringList cells = line.split(QLatin1Char('\t'));
        const QString& kind = cells.at(0);
        if (kind == QLatin1String("title")) {
            title_ = cells.value(1);
            header_->setText(title_);
        } else if (kind == QLatin1String("note")) {
            note_ = new QLabel(cells.value(1));
            note_->setWordWrap(true);
            note_->setEnabled(false);
            controls_->insertWidget(controls_->count() - 1, note_);
        } else if (kind == QLatin1String("groups")) {
            groups_ = new QComboBox();
            groups_->setObjectName(QStringLiteral("propertiesGroup"));
            groups_->addItems(cells.value(1).split(QLatin1Char('|')));
            controls_->insertWidget(controls_->count() - 1, groups_);
            connect(groups_, &QComboBox::currentIndexChanged, this, &PropertiesPanel::showGroup);
        } else if (kind == QLatin1String("curves")) {
            addCurves();
        } else if (kind == QLatin1String("slider")) {
            addSlider(cells);
        } else if (kind == QLatin1String("check")) {
            addCheck(cells);
        } else if (kind == QLatin1String("choice")) {
            addChoice(cells);
        } else if (kind == QLatin1String("color")) {
            addColor(cells);
        }
    }
    showGroup(groups_ ? groups_->currentIndex() : 0);
    loading_ = false;
}

void PropertiesPanel::loadAdjustment(const QStringList& page)
{
    loading_ = true;
    for (const QString& line : page) {
        const QStringList cells = line.split(QLatin1Char('\t'));
        for (Row& row : rows_) {
            if (cells.size() > 4 && row.key == cells.at(1) && row.load) {
                row.load(cells.at(4).toDouble());
            }
        }
    }
    if (curve_) {
        loadCurve();
    }
    loading_ = false;
}

void PropertiesPanel::addSlider(const QStringList& cells)
{
    const QString key = cells.value(1);
    const int decimals = cells.value(8).toInt();
    const double factor = std::pow(10.0, decimals);
    auto* container = new QWidget();
    auto* row = new QHBoxLayout(container);
    row->setContentsMargins(0, 0, 0, 0);
    row->addWidget(new QLabel(cells.value(2) + QLatin1Char(':'), container));
    auto* spin = new QDoubleSpinBox(container);
    spin->setDecimals(decimals);
    spin->setRange(cells.value(6).toDouble(), cells.value(7).toDouble());
    spin->setKeyboardTracking(false);
    auto* slider = new QSlider(Qt::Horizontal, container);
    slider->setObjectName(QStringLiteral("propertiesSlider.") + key);
    slider->setRange(qRound(spin->minimum() * factor), qRound(spin->maximum() * factor));
    auto* column = new QVBoxLayout();
    row->addStretch(1);
    row->addWidget(spin);
    column->addWidget(container);
    column->addWidget(slider);
    auto* holder = new QWidget();
    holder->setLayout(column);
    column->setContentsMargins(0, 0, 0, 4);
    controls_->insertWidget(controls_->count() - 1, holder);

    const auto load = [slider, spin, factor](double v) {
        const QSignalBlocker a(slider);
        const QSignalBlocker b(spin);
        spin->setValue(v);
        slider->setValue(qRound(v * factor));
    };
    load(cells.value(4).toDouble());
    connect(slider, &QSlider::valueChanged, this, [this, key, spin, factor](int v) {
        const QSignalBlocker block(spin);
        spin->setValue(v / factor);
        push(key, v / factor);
    });
    connect(slider, &QSlider::sliderReleased, this, &PropertiesPanel::commit);
    connect(spin, &QDoubleSpinBox::valueChanged, this, [this, key, slider, factor](double v) {
        const QSignalBlocker block(slider);
        slider->setValue(qRound(v * factor));
        push(key, v);
    });
    rows_.push_back({key, cells.value(3).toInt(), holder, slider, load});
}

void PropertiesPanel::addCheck(const QStringList& cells)
{
    const QString key = cells.value(1);
    auto* box = new QCheckBox(cells.value(2));
    box->setObjectName(QStringLiteral("propertiesCheck.") + key);
    const auto load = [box](double v) {
        const QSignalBlocker block(box);
        box->setChecked(v != 0.0);
    };
    load(cells.value(4).toDouble());
    connect(box, &QCheckBox::toggled, this, [this, key](bool on) {
        push(key, on ? 1.0 : 0.0);
        commit();
    });
    controls_->insertWidget(controls_->count() - 1, box);
    rows_.push_back({key, cells.value(3).toInt(), box, box, load});
}

void PropertiesPanel::addChoice(const QStringList& cells)
{
    const QString key = cells.value(1);
    auto* container = new QWidget();
    auto* row = new QHBoxLayout(container);
    row->setContentsMargins(0, 0, 0, 0);
    row->addWidget(new QLabel(cells.value(2) + QLatin1Char(':'), container));
    auto* combo = new QComboBox(container);
    combo->setObjectName(QStringLiteral("propertiesChoice.") + key);
    combo->addItems(cells.value(6).split(QLatin1Char('|')));
    row->addWidget(combo, 1);
    const auto load = [combo](double v) {
        const QSignalBlocker block(combo);
        combo->setCurrentIndex(int(v));
    };
    load(cells.value(4).toDouble());
    connect(combo, &QComboBox::currentIndexChanged, this, [this, key](int i) {
        push(key, i);
        commit();
    });
    controls_->insertWidget(controls_->count() - 1, container);
    rows_.push_back({key, cells.value(3).toInt(), container, combo, load});
}

void PropertiesPanel::addColor(const QStringList& cells)
{
    const QString key = cells.value(1);
    auto* container = new QWidget();
    auto* row = new QHBoxLayout(container);
    row->setContentsMargins(0, 0, 0, 0);
    row->addWidget(new QLabel(cells.value(2) + QLatin1Char(':'), container));
    auto* button = new QToolButton(container);
    button->setObjectName(QStringLiteral("propertiesColor.") + key);
    button->setAutoRaise(true);
    row->addWidget(button);
    row->addStretch(1);
    auto color = std::make_shared<QColor>();
    const auto load = [button, color](double v) {
        *color = QColor::fromRgb(QRgb(uint(v)) | 0xff000000u);
        button->setIcon(swatch(*color));
    };
    load(cells.value(4).toDouble());
    connect(button, &QToolButton::clicked, this, [this, key, color, load]() {
        const QColor picked = ColorPickerDialog::getColor(*color, this, QStringLiteral("Color"));
        if (picked.isValid()) {
            load(double(picked.rgb() & 0xffffffu));
            push(key, double(picked.rgb() & 0xffffffu));
            commit();
        }
    });
    controls_->insertWidget(controls_->count() - 1, container);
    rows_.push_back({key, cells.value(3).toInt(), container, button, load});
}

void PropertiesPanel::addCurves()
{
    curveChannel_ = new QComboBox();
    curveChannel_->setObjectName(QStringLiteral("propertiesCurveChannel"));
    curveChannel_->addItems({QStringLiteral("RGB"), QStringLiteral("Red"), QStringLiteral("Green"),
                             QStringLiteral("Blue")});
    controls_->insertWidget(controls_->count() - 1, curveChannel_);
    curve_ = new CurveWidget();
    curve_->setObjectName(QStringLiteral("propertiesCurve"));
    curve_->setMinimumSize(180, 180);
    curve_->setShowHistogram(false);
    controls_->insertWidget(controls_->count() - 1, curve_);
    connect(curveChannel_, &QComboBox::currentIndexChanged, this, [this]() {
        commit();
        loading_ = true;
        loadCurve();
        loading_ = false;
    });
    connect(curve_, &CurveWidget::curveChanged, this, [this]() {
        if (loading_ || !view_) {
            return;
        }
        // Whole levels, strictly increasing inputs, as the curv block stores.
        QStringList pairs;
        int lastX = -1;
        for (const QPointF& p : curve_->points()) {
            const int x = qBound(0, qRound(p.x() * 255.0), 255);
            const int y = qBound(0, qRound(p.y() * 255.0), 255);
            if (x > lastX) {
                pairs << QStringLiteral("%1,%2").arg(x).arg(y);
                lastX = x;
            }
        }
        if (adjustment_set_curve(*view_, path_, curveChannel_->currentIndex(),
                                 pairs.join(QLatin1Char(' ')))) {
            edited();
        }
    });
    loadCurve();
}

void PropertiesPanel::loadCurve()
{
    if (!view_ || !curve_) {
        return;
    }
    QVector<QPointF> points;
    const QStringList pairs =
        adjustment_curve(*view_, path_, curveChannel_->currentIndex()).split(QLatin1Char(' '));
    for (const QString& pair : pairs) {
        const QStringList xy = pair.split(QLatin1Char(','));
        if (xy.size() == 2) {
            points.append(QPointF(xy[0].toInt() / 255.0, xy[1].toInt() / 255.0));
        }
    }
    const QSignalBlocker block(curve_);
    curve_->setPoints(points);
}

void PropertiesPanel::showGroup(int group)
{
    for (const Row& row : rows_) {
        row.widget->setVisible(row.group < 0 || row.group == group);
    }
}

void PropertiesPanel::push(const QString& key, double value)
{
    if (loading_ || !view_ || path_.isEmpty()) {
        return;
    }
    if (adjustment_set(*view_, path_, key, value)) {
        edited();
    }
}

void PropertiesPanel::edited()
{
    dirty_ = true;
    commitTimer_->start();
}

void PropertiesPanel::commit()
{
    commitTimer_->stop();
    if (!dirty_) {
        return;
    }
    dirty_ = false;
    if (view_) {
        adjustment_commit(*view_, title_);
    }
}

QString PropertiesPanel::messageForTest() const { return header_->text(); }

QWidget* PropertiesPanel::controlForTest(const QString& key) const
{
    for (const Row& row : rows_) {
        if (row.key == key) {
            return row.control;
        }
    }
    if (key == QLatin1String("curve")) {
        return curve_;
    }
    if (key == QLatin1String("group")) {
        return groups_;
    }
    return nullptr;
}

} // namespace pictura
