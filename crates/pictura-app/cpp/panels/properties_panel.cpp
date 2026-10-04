#include "properties_panel.h"

#include "adjustment_controls.h"
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

namespace pictura {

namespace {

// A pause this long after the last live edit (a wheel, arrow keys, or a curve
// drag, none of which has a release to hang the commit on) ends the gesture.
constexpr int kCommitDelayMs = 500;

enum Page { NothingPage, AdjustmentPage, LayerPage };

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

    controls_ = new AdjustmentControls(stack_);
    controls_->setObjectName(QStringLiteral("propertiesControls"));
    controls_->setCurveSource([this](int channel) {
        return view_ ? QString(adjustment_curve(*view_, path_, channel)) : QString();
    });
    connect(controls_, &AdjustmentControls::valueChanged, this, &PropertiesPanel::push);
    connect(controls_, &AdjustmentControls::curveChanged, this,
            [this](int channel, const QString& points) {
                if (view_ && adjustment_set_curve(*view_, path_, channel, points)) {
                    edited();
                }
            });
    connect(controls_, &AdjustmentControls::gestureEnded, this, &PropertiesPanel::commit);
    stack_->addWidget(controls_);

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
    controls_->setPage(page, dirty_);
    title_ = controls_->title();
    header_->setText(title_);
}

void PropertiesPanel::push(const QString& key, double value)
{
    if (!view_ || path_.isEmpty()) {
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
    return controls_->controlForTest(key);
}

} // namespace pictura
