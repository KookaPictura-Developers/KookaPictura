#include "commands.h"

#include <QtGui/QAction>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>

#include <utility>

namespace pictura {

CommandRegistry::CommandRegistry(QObject* parent) : QObject(parent) {}

void CommandRegistry::add(const CommandSpec& spec)
{
    entries_.push_back(Entry{spec, nullptr});
}

void CommandRegistry::add(const QString& id, const QStringList& path, const QString& label,
                          const QKeySequence& shortcut, bool implemented)
{
    CommandSpec spec;
    spec.id = id;
    spec.path = path;
    spec.label = label;
    spec.shortcut = shortcut;
    spec.implemented = implemented;
    entries_.push_back(Entry{spec, nullptr});
}

void CommandRegistry::addSeparator(const QStringList& path)
{
    CommandSpec spec;
    spec.path = path;
    entries_.push_back(Entry{spec, nullptr});
}

void CommandRegistry::setHandler(const QString& id, std::function<void()> handler)
{
    handlers_.insert(id, std::move(handler));
}

bool CommandRegistry::hasHandler(const QString& id) const
{
    return handlers_.contains(id);
}

void CommandRegistry::setEnabledProvider(const QString& id, std::function<bool()> enabled)
{
    enabledProviders_.insert(id, std::move(enabled));
}

void CommandRegistry::setCheckedProvider(const QString& id, std::function<bool()> checked)
{
    checkedProviders_.insert(id, std::move(checked));
}

void CommandRegistry::setLabelProvider(const QString& id, std::function<QString()> label)
{
    labelProviders_.insert(id, std::move(label));
}

QAction* CommandRegistry::action(const QString& id) const
{
    for (const Entry& entry : entries_) {
        if (!entry.spec.id.isEmpty() && entry.spec.id == id) {
            return entry.action;
        }
    }
    return nullptr;
}

void CommandRegistry::buildMenuBar(QMenuBar* menuBar)
{
    menuBar_ = menuBar;
    if (!menuBar_) {
        return;
    }

    menuBar->clear();
    const QList<QMenu*> previous = menuBar->findChildren<QMenu*>(Qt::FindDirectChildrenOnly);
    for (QMenu* menu : previous) {
        delete menu;
    }

    QHash<QString, QMenu*> menus;
    for (Entry& entry : entries_) {
        entry.action = nullptr;
        const QStringList& path = entry.spec.path;
        if (path.isEmpty()) {
            continue;
        }

        const bool separator = entry.spec.id.isEmpty();
        const int menuCount = separator ? path.size() : path.size() - 1;

        QMenu* parent = nullptr;
        QString key;
        for (int i = 0; i < menuCount; ++i) {
            key = (i == 0) ? path.at(0) : key + QLatin1Char('/') + path.at(i);
            QMenu* menu = menus.value(key);
            if (!menu) {
                menu = parent ? parent->addMenu(path.at(i)) : menuBar->addMenu(path.at(i));
                menus.insert(key, menu);
                connect(menu, &QMenu::aboutToShow, this, &CommandRegistry::refresh);
            }
            parent = menu;
        }

        if (!parent) {
            continue;
        }
        if (separator) {
            parent->addSeparator();
            continue;
        }

        QAction* item = parent->addAction(path.last());
        item->setData(entry.spec.id);
        item->setShortcut(entry.spec.shortcut);
        item->setCheckable(entry.spec.checkable);
        const QString id = entry.spec.id;
        connect(item, &QAction::triggered, this, [this, id] { dispatch(id); });
        entry.action = item;
    }

    refresh();
}

void CommandRegistry::refresh()
{
    for (Entry& entry : entries_) {
        if (!entry.action) {
            continue;
        }
        const CommandSpec& spec = entry.spec;

        bool enabled = spec.implemented && hasHandler(spec.id);
        if (enabledProviders_.contains(spec.id)) {
            enabled = enabled && enabledProviders_[spec.id]();
        }
        entry.action->setEnabled(enabled);

        if (spec.checkable) {
            const bool checked = checkedProviders_.contains(spec.id) && checkedProviders_[spec.id]();
            entry.action->setChecked(checked);
        }

        const QString text =
            labelProviders_.contains(spec.id) ? labelProviders_[spec.id]() : spec.label;
        entry.action->setText(text);
    }
}

bool CommandRegistry::dispatch(const QString& id)
{
    auto it = handlers_.find(id);
    if (it == handlers_.end()) {
        return false;
    }
    it.value()();
    return true;
}

QStringList CommandRegistry::topLevelTitles() const
{
    QStringList titles;
    for (const Entry& entry : entries_) {
        if (entry.spec.path.isEmpty()) {
            continue;
        }
        const QString& top = entry.spec.path.first();
        if (!titles.contains(top)) {
            titles.append(top);
        }
    }
    return titles;
}

} // namespace pictura
