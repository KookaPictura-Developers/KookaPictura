#include "layers_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QAbstractTableModel>
#include <QtCore/QItemSelectionModel>
#include <QtCore/QModelIndex>
#include <QtCore/QSize>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtCore/QVariant>
#include <QtCore/QVector>
#include <QtGui/QAction>
#include <QtGui/QImage>
#include <QtWidgets/QAbstractItemView>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QHeaderView>
#include <QtWidgets/QMenu>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QTreeView>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

#include <array>
#include <utility>

namespace pictura {

namespace {

struct BlendEntry {
    const char* key;
    const char* name;
};

constexpr std::array<BlendEntry, 27> kBlends{{
    {"norm", "Normal"},
    {"diss", "Dissolve"},
    {"dark", "Darken"},
    {"mul ", "Multiply"},
    {"idiv", "Color Burn"},
    {"lbrn", "Linear Burn"},
    {"dkCl", "Darker Color"},
    {"lite", "Lighten"},
    {"scrn", "Screen"},
    {"div ", "Color Dodge"},
    {"lddg", "Linear Dodge (Add)"},
    {"lgCl", "Lighter Color"},
    {"over", "Overlay"},
    {"sLit", "Soft Light"},
    {"hLit", "Hard Light"},
    {"vLit", "Vivid Light"},
    {"lLit", "Linear Light"},
    {"pLit", "Pin Light"},
    {"hMix", "Hard Mix"},
    {"diff", "Difference"},
    {"smud", "Exclusion"},
    {"fsub", "Subtract"},
    {"fdiv", "Divide"},
    {"hue ", "Hue"},
    {"sat ", "Saturation"},
    {"col ", "Color"},
    {"lum ", "Luminosity"},
}};

struct LayerRow {
    int index = 0;
    QString name;
    QString kind;
    bool visible = true;
    QString blend;
    int opacity = 255;
    QImage thumbnail;
};

QString blendName(const QString& key)
{
    for (const BlendEntry& entry : kBlends) {
        if (key == QLatin1String(entry.key)) {
            return QString::fromLatin1(entry.name);
        }
    }
    return key;
}

QString layerTooltip(const LayerRow& layer)
{
    if (layer.kind.isEmpty() || layer.kind == QLatin1String("pixel")) {
        return layer.name;
    }
    return QStringLiteral("%1 (%2)").arg(layer.name, layer.kind);
}

} // namespace

// ponytail: blend/opacity are read-only in the model and edited through the
// header controls; add per-column delegates if inline editing is required.
class LayersModel : public QAbstractTableModel {
public:
    explicit LayersModel(QObject* parent = nullptr)
        : QAbstractTableModel(parent)
    {
    }

    void setView(PictureView* view) { view_ = view; }

    void setRows(QVector<LayerRow> rows)
    {
        beginResetModel();
        rows_ = std::move(rows);
        endResetModel();
    }

    const LayerRow& row(int row) const { return rows_[row]; }

    int rowCount(const QModelIndex& parent = QModelIndex()) const override
    {
        return parent.isValid() ? 0 : rows_.size();
    }

    int columnCount(const QModelIndex& parent = QModelIndex()) const override
    {
        return parent.isValid() ? 0 : 5;
    }

    QVariant data(const QModelIndex& index, int role) const override
    {
        if (!index.isValid() || index.row() >= rows_.size()) {
            return {};
        }
        const LayerRow& layer = rows_[index.row()];
        switch (index.column()) {
        case 0:
            if (role == Qt::CheckStateRole) {
                return layer.visible ? Qt::Checked : Qt::Unchecked;
            }
            break;
        case 1:
            if (role == Qt::DecorationRole) {
                return layer.thumbnail;
            }
            break;
        case 2:
            if (role == Qt::DisplayRole || role == Qt::EditRole) {
                return layer.name;
            }
            if (role == Qt::ToolTipRole) {
                return layerTooltip(layer);
            }
            break;
        case 3:
            if (role == Qt::DisplayRole) {
                return blendName(layer.blend);
            }
            break;
        case 4:
            if (role == Qt::DisplayRole) {
                return QString::number(layer.opacity);
            }
            break;
        default:
            break;
        }
        return {};
    }

    Qt::ItemFlags flags(const QModelIndex& index) const override
    {
        if (!index.isValid()) {
            return Qt::NoItemFlags;
        }
        Qt::ItemFlags flags = Qt::ItemIsEnabled | Qt::ItemIsSelectable;
        if (index.column() == 0) {
            flags |= Qt::ItemIsUserCheckable;
        } else if (index.column() == 2) {
            flags |= Qt::ItemIsEditable;
        }
        return flags;
    }

    bool setData(const QModelIndex& index, const QVariant& value, int role) override
    {
        if (!index.isValid() || index.row() >= rows_.size()) {
            return false;
        }
        if (!view_) {
            return false;
        }
        const LayerRow& layer = rows_[index.row()];
        if (role == Qt::CheckStateRole && index.column() == 0) {
            view_->set_layer_visible(layer.index, value.toInt() == Qt::Checked);
            return true;
        }
        if (role == Qt::EditRole && index.column() == 2) {
            const QString name = value.toString().trimmed();
            if (name.isEmpty() || name == layer.name) {
                return false;
            }
            return view_->set_layer_name(layer.index, name);
        }
        return false;
    }

    QVariant headerData(int section, Qt::Orientation orientation, int role) const override
    {
        if (orientation != Qt::Horizontal || role != Qt::DisplayRole) {
            return {};
        }
        switch (section) {
        case 0:
            return QString();
        case 1:
            return QStringLiteral("Thumbnail");
        case 2:
            return QStringLiteral("Name");
        case 3:
            return QStringLiteral("Mode");
        case 4:
            return QStringLiteral("Opacity");
        default:
            return {};
        }
    }

private:
    PictureView* view_ = nullptr;
    QVector<LayerRow> rows_;
};

LayersPanel::LayersPanel(QWidget* parent)
    : QDockWidget(tr("Layers"), parent)
{
    auto* body = new QWidget(this);
    auto* layout = new QVBoxLayout(body);

    auto* controls = new QHBoxLayout();
    blend_ = new QComboBox(body);
    for (const BlendEntry& entry : kBlends) {
        blend_->addItem(QString::fromLatin1(entry.name), QString::fromLatin1(entry.key));
    }
    blend_->setEnabled(false);
    opacity_ = new QSpinBox(body);
    opacity_->setRange(0, 255);
    opacity_->setEnabled(false);
    controls->addWidget(blend_, 1);
    controls->addWidget(opacity_);
    layout->addLayout(controls);

    model_ = new LayersModel(this);

    tree_ = new QTreeView(body);
    tree_->setModel(model_);
    tree_->setRootIsDecorated(false);
    tree_->setItemsExpandable(false);
    tree_->setAllColumnsShowFocus(true);
    tree_->setSelectionBehavior(QAbstractItemView::SelectRows);
    tree_->setSelectionMode(QAbstractItemView::SingleSelection);
    tree_->setIconSize(QSize(24, 24));
    tree_->header()->setSectionResizeMode(2, QHeaderView::Stretch);
    layout->addWidget(tree_, 1);

    auto* buttons = new QHBoxLayout();
    auto* addButton = new QToolButton(body);
    addButton->setText(tr("Add Adjustment"));
    addButton->setPopupMode(QToolButton::InstantPopup);
    addButton->setToolButtonStyle(Qt::ToolButtonTextOnly);
    auto* menu = new QMenu(addButton);
    const QStringList kinds = {
        QStringLiteral("invert"),
        QStringLiteral("posterize"),
        QStringLiteral("threshold"),
        QStringLiteral("brightness-contrast"),
        QStringLiteral("hue-saturation"),
    };
    for (const QString& kind : kinds) {
        QAction* action = menu->addAction(kind);
        connect(action, &QAction::triggered, this, [this, kind] {
            if (view_) {
                view_->add_adjustment(kind);
            }
        });
    }
    addButton->setMenu(menu);
    auto* deleteButton = new QPushButton(tr("Delete Layer"), body);
    auto* upButton = new QPushButton(tr("Move Up"), body);
    auto* downButton = new QPushButton(tr("Move Down"), body);
    buttons->addWidget(addButton);
    buttons->addWidget(deleteButton);
    buttons->addWidget(upButton);
    buttons->addWidget(downButton);
    layout->addLayout(buttons);

    setWidget(body);

    connect(tree_->selectionModel(), &QItemSelectionModel::currentChanged, this,
            [this](const QModelIndex&, const QModelIndex&) { syncControls(); });
    connect(blend_, &QComboBox::currentIndexChanged, this, [this](int index) {
        if (syncing_ || !view_) {
            return;
        }
        const QModelIndex current = tree_->currentIndex();
        if (current.isValid()) {
            view_->set_layer_blend(model_->row(current.row()).index, blend_->itemData(index).toString());
        }
    });
    connect(opacity_, &QSpinBox::valueChanged, this, [this](int value) {
        if (syncing_ || !view_) {
            return;
        }
        const QModelIndex current = tree_->currentIndex();
        if (current.isValid()) {
            view_->set_layer_opacity(model_->row(current.row()).index, value);
        }
    });
    connect(deleteButton, &QPushButton::clicked, this, [this] {
        const QModelIndex current = tree_->currentIndex();
        if (view_ && current.isValid()) {
            view_->remove_layer(model_->row(current.row()).index);
        }
    });
    connect(upButton, &QPushButton::clicked, this, [this] {
        const QModelIndex current = tree_->currentIndex();
        if (view_ && current.isValid()) {
            view_->move_layer(model_->row(current.row()).index, +1);
        }
    });
    connect(downButton, &QPushButton::clicked, this, [this] {
        const QModelIndex current = tree_->currentIndex();
        if (view_ && current.isValid()) {
            view_->move_layer(model_->row(current.row()).index, -1);
        }
    });
}

void LayersPanel::setView(PictureView* view)
{
    if (viewConnection_) {
        QObject::disconnect(viewConnection_);
    }
    view_ = view;
    model_->setView(view);
    if (view_) {
        viewConnection_ = connect(view_, &PictureView::changed, this, [this] {
            QTimer::singleShot(0, this, [this] { refresh(); });
        });
    }
    refresh();
}

void LayersPanel::refresh()
{
    int selectedLayer = -1;
    const QModelIndex current = tree_->currentIndex();
    if (current.isValid()) {
        selectedLayer = model_->row(current.row()).index;
    }

    QVector<LayerRow> rows;
    if (view_) {
        const int count = view_->layer_count();
        rows.reserve(count);
        for (int i = count - 1; i >= 0; --i) {
            LayerRow layer;
            layer.index = i;
            layer.name = view_->layer_name(i);
            layer.kind = view_->layer_kind(i);
            layer.visible = view_->layer_visible(i);
            layer.blend = view_->layer_blend(i);
            layer.opacity = view_->layer_opacity(i);
            layer.thumbnail = view_->layer_thumbnail(i, 24);
            rows.push_back(layer);
        }
    }
    model_->setRows(std::move(rows));

    if (selectedLayer >= 0) {
        for (int r = 0; r < model_->rowCount(); ++r) {
            if (model_->row(r).index == selectedLayer) {
                tree_->setCurrentIndex(model_->index(r, 0));
                break;
            }
        }
    }
    syncControls();
}

void LayersPanel::syncControls()
{
    const QModelIndex current = tree_->currentIndex();
    const bool active = view_ && current.isValid() && current.row() < model_->rowCount();
    syncing_ = true;
    blend_->setEnabled(active);
    opacity_->setEnabled(active);
    if (active) {
        const LayerRow& layer = model_->row(current.row());
        const int blend = blend_->findData(layer.blend);
        blend_->setCurrentIndex(blend >= 0 ? blend : 0);
        opacity_->setValue(layer.opacity);
    }
    syncing_ = false;
}

} // namespace pictura
