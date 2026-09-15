#include "dialogs.h"

#include <QtWidgets/QMessageBox>
#include <QtWidgets/QPushButton>

namespace pictura {

namespace {

bool g_interactive = true;
UnsavedChoice g_nonInteractiveChoice = UnsavedChoice::Discard;

} // namespace

void setUnsavedPromptInteractive(bool interactive)
{
    g_interactive = interactive;
}

void setNonInteractiveUnsavedChoice(UnsavedChoice choice)
{
    g_nonInteractiveChoice = choice;
}

UnsavedChoice askUnsaved(QWidget* parent, const QString& documentName)
{
    if (!g_interactive) {
        return g_nonInteractiveChoice;
    }

    QMessageBox box(QMessageBox::Warning, QStringLiteral("Kooka Pictura"),
                    QStringLiteral("Save changes to \"%1\"?").arg(documentName),
                    QMessageBox::NoButton, parent);
    QPushButton* saveButton = box.addButton(QStringLiteral("Save"), QMessageBox::AcceptRole);
    box.addButton(QStringLiteral("Discard"), QMessageBox::DestructiveRole);
    QPushButton* cancelButton = box.addButton(QStringLiteral("Cancel"), QMessageBox::RejectRole);
    box.setDefaultButton(saveButton);
    box.exec();

    const QAbstractButton* clicked = box.clickedButton();
    if (clicked == saveButton) {
        return UnsavedChoice::Save;
    }
    if (clicked == cancelButton || !clicked) {
        return UnsavedChoice::Cancel;
    }
    return UnsavedChoice::Discard;
}

} // namespace pictura
