#pragma once

#include <QtWidgets/QWidget>

class QButtonGroup;

namespace pictura {

class ToolController;

// Window > Paragraph: the type tools' paragraph alignment (left / centre /
// right; top / centre / bottom while a vertical Type tool is active), shared
// with the Type options bar. Ported from photorust's ParagraphPanel.
// ponytail: the type model has no justification, indents, paragraph spacing,
// or hyphenation (point text does not wrap); they are shown disabled.
class ParagraphPanel : public QWidget {
    Q_OBJECT

public:
    explicit ParagraphPanel(QWidget* parent = nullptr);

    void setController(ToolController* controller);
    // Re-read the alignment and the active tool's orientation.
    void refresh();

private:
    ToolController* controller_ = nullptr;
    QButtonGroup* align_ = nullptr;
    bool vertical_ = false;
};

} // namespace pictura
