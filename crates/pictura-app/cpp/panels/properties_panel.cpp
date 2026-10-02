#include "properties_panel.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtWidgets/QLabel>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

PropertiesPanel::PropertiesPanel(QWidget* parent)
    : QWidget(parent)
{
    auto* layout = new QVBoxLayout(this);
    message_ = new QLabel(this);
    message_->setAlignment(Qt::AlignCenter);
    message_->setEnabled(false);
    layout->addWidget(message_, 1, Qt::AlignCenter);
    refresh();
}

void PropertiesPanel::setView(PictureView* view)
{
    view_ = view;
    refresh();
}

void PropertiesPanel::refresh()
{
    QString message = QStringLiteral("No Properties");
    if (view_ && view_->has_document()) {
        const QString active = view_->active_layer_path();
        if (!active.isEmpty()) {
            const int rows = view_->layer_row_count();
            for (int i = 0; i < rows; ++i) {
                if (view_->layer_row_path(i) == active
                    && view_->layer_row_has_adjustment(i)) {
                    message = view_->layer_row_name(i);
                    break;
                }
            }
        }
    }
    // ponytail: display-only — there is no adjustment-parameter get/set bridge
    // and no edit session, so an adjustment stays opaque `AdjustmentData`.
    // Add editing when a parameter bridge and edit session land.
    message_->setText(message);
}

QString PropertiesPanel::messageForTest() const { return message_->text(); }

} // namespace pictura
