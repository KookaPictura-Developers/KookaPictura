#pragma once

#include <QtWidgets/QWidget>

class QComboBox;
class QFontComboBox;
class QToolButton;

namespace pictura {

class ToolController;

// Window > Character: the type tools' shared options — font family, size
// (px), text colour, and anti-aliasing — edited in place, as the Type options
// bar edits them; a type layer being edited or selected takes the change.
// Ported from photorust's CharacterPanel.
// ponytail: the type model has no font style, leading, tracking, kerning,
// baseline shift, or horizontal / vertical scale; they are shown disabled, and
// anti-aliasing offers only None / Sharp.
class CharacterPanel : public QWidget {
    Q_OBJECT

public:
    explicit CharacterPanel(QWidget* parent = nullptr);

    void setController(ToolController* controller);
    // Re-read the type options into the controls.
    void refresh();

private:
    void applySize();

    ToolController* controller_ = nullptr;
    QFontComboBox* family_ = nullptr;
    QComboBox* size_ = nullptr;
    QToolButton* color_ = nullptr;
    QComboBox* antialias_ = nullptr;
};

} // namespace pictura
