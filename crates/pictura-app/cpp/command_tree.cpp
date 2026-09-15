// Command table for the documented CS6 menu tree (docs/02-ui-ux/menus.md).
#include "commands.h"

namespace pictura {
namespace {

// ponytail: unimplemented leaves get path-derived ids; swap in frozen
// command_ids once each gains a handler.
QString idFor(const QStringList& path) {
    QString id;
    for (const QString& segment : path) {
        for (const QChar c : segment) {
            if (c.isLetterOrNumber()) {
                id += c.toLower();
            } else if (c == QLatin1Char(' ') || c == QLatin1Char('-')) {
                id += QLatin1Char('.');
            }
        }
        id += QLatin1Char('.');
    }
    id.chop(1);
    return id;
}

void leaf(CommandRegistry& registry, const QStringList& path, const QString& label,
          const QString& shortcut = QString()) {
    registry.add(idFor(path), path, label,
                 shortcut.isEmpty() ? QKeySequence() : QKeySequence(shortcut));
}

} // namespace

void addDefaultCommands(CommandRegistry& registry) {
    // File
    registry.add(command_ids::FileNew, {"File", "New…"}, QStringLiteral("New…"),
                 QKeySequence(QStringLiteral("Ctrl+N")), true);
    registry.add(command_ids::FileOpen, {"File", "Open..."}, QStringLiteral("Open..."),
                 QKeySequence(QStringLiteral("Ctrl+O")), true);
    leaf(registry, {"File", "Open As…"}, QStringLiteral("Open As…"),
         QStringLiteral("Ctrl+Alt+Shift+O"));
    leaf(registry, {"File", "Open As Smart Object…"}, QStringLiteral("Open As Smart Object…"));
    registry.addSeparator({"File"});
    registry.add(command_ids::FileClose, {"File", "Close"}, QStringLiteral("Close"),
                 QKeySequence(QStringLiteral("Ctrl+W")), true);
    registry.add(command_ids::FileCloseAll, {"File", "Close All"}, QStringLiteral("Close All"),
                 QKeySequence(QStringLiteral("Ctrl+Alt+W")), true);
    leaf(registry, {"File", "Close and Go to Bridge…"}, QStringLiteral("Close and Go to Bridge…"),
         QStringLiteral("Shift+Ctrl+W"));
    registry.addSeparator({"File"});
    registry.add(command_ids::FileSave, {"File", "Save"}, QStringLiteral("Save"),
                 QKeySequence(QStringLiteral("Ctrl+S")), true);
    registry.add(command_ids::FileSaveAs, {"File", "Save As…"}, QStringLiteral("Save As…"),
                 QKeySequence(QStringLiteral("Shift+Ctrl+S")), true);
    leaf(registry, {"File", "Check In…"}, QStringLiteral("Check In…"));
    leaf(registry, {"File", "Save for Web & Devices…"}, QStringLiteral("Save for Web & Devices…"),
         QStringLiteral("Ctrl+Alt+Shift+S"));
    registry.add(command_ids::FileRevert, {"File", "Revert…"}, QStringLiteral("Revert…"),
                 QKeySequence(QStringLiteral("F12")), true);
    registry.addSeparator({"File"});
    leaf(registry, {"File", "Place…"}, QStringLiteral("Place…"));
    leaf(registry, {"File", "Import", "Variable Data Sets"}, QStringLiteral("Variable Data Sets"));
    leaf(registry, {"File", "Import", "WIA Support"}, QStringLiteral("WIA Support"));
    leaf(registry, {"File", "Export", "Data Sets As Files…"}, QStringLiteral("Data Sets As Files…"));
    leaf(registry, {"File", "Export", "Render Video…"}, QStringLiteral("Render Video…"));
    leaf(registry, {"File", "Export", "Zoomify…"}, QStringLiteral("Zoomify…"));
    leaf(registry, {"File", "Automate", "Batch…"}, QStringLiteral("Batch…"));
    leaf(registry, {"File", "Automate", "Conditional Mode Change…"},
         QStringLiteral("Conditional Mode Change…"));
    leaf(registry, {"File", "Automate", "Contact Sheet II"}, QStringLiteral("Contact Sheet II"));
    leaf(registry, {"File", "Automate", "Create Droplet…"}, QStringLiteral("Create Droplet…"));
    leaf(registry, {"File", "Automate", "Crop And Straighten Photos"},
         QStringLiteral("Crop And Straighten Photos"));
    leaf(registry, {"File", "Automate", "Merge To HDR Pro…"}, QStringLiteral("Merge To HDR Pro…"));
    leaf(registry, {"File", "Automate", "PDF Presentation"}, QStringLiteral("PDF Presentation"));
    leaf(registry, {"File", "Automate", "Photomerge…"}, QStringLiteral("Photomerge…"));
    leaf(registry, {"File", "Automate", "Picture Package"}, QStringLiteral("Picture Package"));
    leaf(registry, {"File", "Scripts", "Export Layers to Files"},
         QStringLiteral("Export Layers to Files"));
    leaf(registry, {"File", "Scripts", "Image Processor"}, QStringLiteral("Image Processor"));
    leaf(registry, {"File", "Scripts", "Layer Comps to Files"}, QStringLiteral("Layer Comps to Files"));
    leaf(registry, {"File", "Scripts", "Load Files into Stack"}, QStringLiteral("Load Files into Stack"));
    leaf(registry, {"File", "Scripts", "Script Events Manager"}, QStringLiteral("Script Events Manager"));
    leaf(registry, {"File", "Scripts", "Statistics"}, QStringLiteral("Statistics"));
    registry.addSeparator({"File"});
    leaf(registry, {"File", "File Info…"}, QStringLiteral("File Info…"),
         QStringLiteral("Ctrl+Alt+Shift+I"));
    leaf(registry, {"File", "Print…"}, QStringLiteral("Print…"), QStringLiteral("Ctrl+P"));
    leaf(registry, {"File", "Print One Copy"}, QStringLiteral("Print One Copy"),
         QStringLiteral("Ctrl+Alt+Shift+P"));
    registry.addSeparator({"File"});
    registry.add(command_ids::FileExit, {"File", "Exit"}, QStringLiteral("Exit"),
                 QKeySequence(QStringLiteral("Ctrl+Q")), true);

    // Edit
    registry.add(command_ids::EditUndo, {"Edit", "Undo"}, QStringLiteral("Undo"),
                 QKeySequence(QStringLiteral("Ctrl+Z")), true);
    registry.add(command_ids::EditRedo, {"Edit", "Redo"}, QStringLiteral("Redo"),
                 QKeySequence(QStringLiteral("Ctrl+Y")), true);
    registry.add(command_ids::EditStepForward, {"Edit", "Step Forward"},
                 QStringLiteral("Step Forward"), QKeySequence(QStringLiteral("Ctrl+Shift+Z")), true);
    registry.add(command_ids::EditStepBackward, {"Edit", "Step Backward"},
                 QStringLiteral("Step Backward"), QKeySequence(QStringLiteral("Ctrl+Alt+Z")), true);
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Fade…"}, QStringLiteral("Fade…"), QStringLiteral("Shift+Ctrl+F"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Cut"}, QStringLiteral("Cut"), QStringLiteral("Ctrl+X"));
    leaf(registry, {"Edit", "Copy"}, QStringLiteral("Copy"), QStringLiteral("Ctrl+C"));
    leaf(registry, {"Edit", "Copy Merged"}, QStringLiteral("Copy Merged"),
         QStringLiteral("Shift+Ctrl+C"));
    leaf(registry, {"Edit", "Paste"}, QStringLiteral("Paste"), QStringLiteral("Ctrl+V"));
    leaf(registry, {"Edit", "Paste Special", "Paste Into"}, QStringLiteral("Paste Into"),
         QStringLiteral("Shift+Ctrl+V"));
    leaf(registry, {"Edit", "Paste Special", "Paste Outside"}, QStringLiteral("Paste Outside"));
    leaf(registry, {"Edit", "Clear"}, QStringLiteral("Clear"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Check Spelling…"}, QStringLiteral("Check Spelling…"));
    leaf(registry, {"Edit", "Find And Replace Text…"}, QStringLiteral("Find And Replace Text…"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Fill…"}, QStringLiteral("Fill…"), QStringLiteral("Shift+F5"));
    leaf(registry, {"Edit", "Stroke…"}, QStringLiteral("Stroke…"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Content-Aware Scale"}, QStringLiteral("Content-Aware Scale"),
         QStringLiteral("Alt+Shift+Ctrl+C"));
    leaf(registry, {"Edit", "Puppet Warp"}, QStringLiteral("Puppet Warp"));
    leaf(registry, {"Edit", "Free Transform"}, QStringLiteral("Free Transform"),
         QStringLiteral("Ctrl+T"));
    leaf(registry, {"Edit", "Transform", "Again"}, QStringLiteral("Again"),
         QStringLiteral("Shift+Ctrl+T"));
    leaf(registry, {"Edit", "Transform", "Scale"}, QStringLiteral("Scale"));
    leaf(registry, {"Edit", "Transform", "Rotate"}, QStringLiteral("Rotate"));
    leaf(registry, {"Edit", "Transform", "Skew"}, QStringLiteral("Skew"));
    leaf(registry, {"Edit", "Transform", "Distort"}, QStringLiteral("Distort"));
    leaf(registry, {"Edit", "Transform", "Perspective"}, QStringLiteral("Perspective"));
    leaf(registry, {"Edit", "Transform", "Warp"}, QStringLiteral("Warp"));
    leaf(registry, {"Edit", "Transform", "Rotate 90° CW"}, QStringLiteral("Rotate 90° CW"));
    leaf(registry, {"Edit", "Transform", "Rotate 90° CCW"}, QStringLiteral("Rotate 90° CCW"));
    leaf(registry, {"Edit", "Transform", "Rotate 180°"}, QStringLiteral("Rotate 180°"));
    leaf(registry, {"Edit", "Transform", "Flip Horizontal"}, QStringLiteral("Flip Horizontal"));
    leaf(registry, {"Edit", "Transform", "Flip Vertical"}, QStringLiteral("Flip Vertical"));
    leaf(registry, {"Edit", "Transform Path", "Again"}, QStringLiteral("Again"));
    leaf(registry, {"Edit", "Transform Path", "Scale"}, QStringLiteral("Scale"));
    leaf(registry, {"Edit", "Transform Path", "Rotate"}, QStringLiteral("Rotate"));
    leaf(registry, {"Edit", "Transform Path", "Skew"}, QStringLiteral("Skew"));
    leaf(registry, {"Edit", "Transform Path", "Distort"}, QStringLiteral("Distort"));
    leaf(registry, {"Edit", "Transform Path", "Perspective"}, QStringLiteral("Perspective"));
    leaf(registry, {"Edit", "Transform Path", "Warp"}, QStringLiteral("Warp"));
    leaf(registry, {"Edit", "Transform Points", "Again"}, QStringLiteral("Again"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Define Brush Preset…"}, QStringLiteral("Define Brush Preset…"));
    leaf(registry, {"Edit", "Define Pattern…"}, QStringLiteral("Define Pattern…"));
    leaf(registry, {"Edit", "Define Custom Shape…"}, QStringLiteral("Define Custom Shape…"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Purge", "Undo"}, QStringLiteral("Undo"));
    leaf(registry, {"Edit", "Purge", "Histories"}, QStringLiteral("Histories"));
    leaf(registry, {"Edit", "Purge", "Clipboard"}, QStringLiteral("Clipboard"));
    leaf(registry, {"Edit", "Purge", "Video Cache"}, QStringLiteral("Video Cache"));
    leaf(registry, {"Edit", "Purge", "All"}, QStringLiteral("All"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Adobe PDF Presets…"}, QStringLiteral("Adobe PDF Presets…"));
    leaf(registry, {"Edit", "Presets", "Preset Manager…"}, QStringLiteral("Preset Manager…"));
    leaf(registry, {"Edit", "Presets", "Migrate Presets…"}, QStringLiteral("Migrate Presets…"));
    leaf(registry, {"Edit", "Presets", "Export Presets…"}, QStringLiteral("Export Presets…"));
    leaf(registry, {"Edit", "Presets", "Import Presets…"}, QStringLiteral("Import Presets…"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Color Settings…"}, QStringLiteral("Color Settings…"),
         QStringLiteral("Shift+Ctrl+K"));
    leaf(registry, {"Edit", "Assign Profile…"}, QStringLiteral("Assign Profile…"));
    leaf(registry, {"Edit", "Convert to Profile…"}, QStringLiteral("Convert to Profile…"));
    registry.addSeparator({"Edit"});
    leaf(registry, {"Edit", "Keyboard Shortcuts…"}, QStringLiteral("Keyboard Shortcuts…"),
         QStringLiteral("Alt+Shift+Ctrl+K"));
    leaf(registry, {"Edit", "Menus…"}, QStringLiteral("Menus…"));
    leaf(registry, {"Edit", "Preferences", "General"}, QStringLiteral("General"),
         QStringLiteral("Ctrl+K"));
    leaf(registry, {"Edit", "Preferences", "Interface"}, QStringLiteral("Interface"));
    leaf(registry, {"Edit", "Preferences", "File Handling"}, QStringLiteral("File Handling"));
    leaf(registry, {"Edit", "Preferences", "Performance"}, QStringLiteral("Performance"));
    leaf(registry, {"Edit", "Preferences", "Cursors"}, QStringLiteral("Cursors"));
    leaf(registry, {"Edit", "Preferences", "Transparency & Gamut"},
         QStringLiteral("Transparency & Gamut"));
    leaf(registry, {"Edit", "Preferences", "Units & Rulers"}, QStringLiteral("Units & Rulers"));
    leaf(registry, {"Edit", "Preferences", "Guides, Grid, & Slices"},
         QStringLiteral("Guides, Grid, & Slices"));
    leaf(registry, {"Edit", "Preferences", "Plug-ins"}, QStringLiteral("Plug-ins"));
    leaf(registry, {"Edit", "Preferences", "Type"}, QStringLiteral("Type"));
    leaf(registry, {"Edit", "Preferences", "3D"}, QStringLiteral("3D"));

    // Image
    leaf(registry, {"Image", "Mode", "Bitmap"}, QStringLiteral("Bitmap"));
    leaf(registry, {"Image", "Mode", "Grayscale"}, QStringLiteral("Grayscale"));
    leaf(registry, {"Image", "Mode", "Duotone"}, QStringLiteral("Duotone"));
    leaf(registry, {"Image", "Mode", "Indexed Color"}, QStringLiteral("Indexed Color"));
    leaf(registry, {"Image", "Mode", "RGB Color"}, QStringLiteral("RGB Color"));
    leaf(registry, {"Image", "Mode", "CMYK Color"}, QStringLiteral("CMYK Color"));
    leaf(registry, {"Image", "Mode", "Lab Color"}, QStringLiteral("Lab Color"));
    leaf(registry, {"Image", "Mode", "Multichannel"}, QStringLiteral("Multichannel"));
    leaf(registry, {"Image", "Mode", "Color Table"}, QStringLiteral("Color Table"));
    registry.addSeparator({"Image", "Mode"});
    leaf(registry, {"Image", "Mode", "8 Bits/Channel"}, QStringLiteral("8 Bits/Channel"));
    leaf(registry, {"Image", "Mode", "16 Bits/Channel"}, QStringLiteral("16 Bits/Channel"));
    leaf(registry, {"Image", "Mode", "32 Bits/Channel"}, QStringLiteral("32 Bits/Channel"));
    leaf(registry, {"Image", "Adjustments", "Brightness/Contrast"},
         QStringLiteral("Brightness/Contrast"));
    leaf(registry, {"Image", "Adjustments", "Levels"}, QStringLiteral("Levels"));
    leaf(registry, {"Image", "Adjustments", "Curves"}, QStringLiteral("Curves"));
    leaf(registry, {"Image", "Adjustments", "Exposure"}, QStringLiteral("Exposure"));
    leaf(registry, {"Image", "Adjustments", "Vibrance"}, QStringLiteral("Vibrance"));
    leaf(registry, {"Image", "Adjustments", "Hue/Saturation"}, QStringLiteral("Hue/Saturation"));
    leaf(registry, {"Image", "Adjustments", "Color Balance"}, QStringLiteral("Color Balance"));
    leaf(registry, {"Image", "Adjustments", "Black & White"}, QStringLiteral("Black & White"));
    leaf(registry, {"Image", "Adjustments", "Photo Filter"}, QStringLiteral("Photo Filter"));
    leaf(registry, {"Image", "Adjustments", "Channel Mixer"}, QStringLiteral("Channel Mixer"));
    leaf(registry, {"Image", "Adjustments", "Color Lookup"}, QStringLiteral("Color Lookup"));
    leaf(registry, {"Image", "Adjustments", "Invert"}, QStringLiteral("Invert"));
    leaf(registry, {"Image", "Adjustments", "Posterize"}, QStringLiteral("Posterize"));
    leaf(registry, {"Image", "Adjustments", "Threshold"}, QStringLiteral("Threshold"));
    leaf(registry, {"Image", "Adjustments", "Gradient Map"}, QStringLiteral("Gradient Map"));
    leaf(registry, {"Image", "Adjustments", "Selective Color"}, QStringLiteral("Selective Color"));
    leaf(registry, {"Image", "Adjustments", "Shadows/Highlights"},
         QStringLiteral("Shadows/Highlights"));
    leaf(registry, {"Image", "Adjustments", "HDR Toning"}, QStringLiteral("HDR Toning"));
    leaf(registry, {"Image", "Adjustments", "Desaturate"}, QStringLiteral("Desaturate"));
    leaf(registry, {"Image", "Adjustments", "Match Color"}, QStringLiteral("Match Color"));
    leaf(registry, {"Image", "Adjustments", "Replace Color"}, QStringLiteral("Replace Color"));
    leaf(registry, {"Image", "Adjustments", "Equalize"}, QStringLiteral("Equalize"));
    leaf(registry, {"Image", "Adjustments", "Auto Tone"}, QStringLiteral("Auto Tone"));
    leaf(registry, {"Image", "Adjustments", "Auto Contrast"}, QStringLiteral("Auto Contrast"));
    leaf(registry, {"Image", "Adjustments", "Auto Color"}, QStringLiteral("Auto Color"));
    registry.addSeparator({"Image"});
    leaf(registry, {"Image", "Auto Tone"}, QStringLiteral("Auto Tone"), QStringLiteral("Shift+Ctrl+L"));
    leaf(registry, {"Image", "Auto Contrast"}, QStringLiteral("Auto Contrast"),
         QStringLiteral("Alt+Shift+Ctrl+L"));
    leaf(registry, {"Image", "Auto Color"}, QStringLiteral("Auto Color"), QStringLiteral("Shift+Ctrl+B"));
    registry.addSeparator({"Image"});
    leaf(registry, {"Image", "Image Size…"}, QStringLiteral("Image Size…"),
         QStringLiteral("Ctrl+Alt+I"));
    leaf(registry, {"Image", "Canvas Size…"}, QStringLiteral("Canvas Size…"),
         QStringLiteral("Ctrl+Alt+C"));
    registry.addSeparator({"Image"});
    registry.add(command_ids::ImageRotate180, {"Image", "Image Rotation", "180°"},
                 QStringLiteral("180°"), QKeySequence(), true);
    registry.add(command_ids::ImageRotate90Cw, {"Image", "Image Rotation", "90° CW"},
                 QStringLiteral("90° CW"), QKeySequence(), true);
    registry.add(command_ids::ImageRotate90Ccw, {"Image", "Image Rotation", "90° CCW"},
                 QStringLiteral("90° CCW"), QKeySequence(), true);
    leaf(registry, {"Image", "Image Rotation", "Arbitrary…"}, QStringLiteral("Arbitrary…"));
    registry.add(command_ids::ImageFlipHorizontal, {"Image", "Image Rotation", "Flip Canvas Horizontal"},
                 QStringLiteral("Flip Canvas Horizontal"), QKeySequence(), true);
    registry.add(command_ids::ImageFlipVertical, {"Image", "Image Rotation", "Flip Canvas Vertical"},
                 QStringLiteral("Flip Canvas Vertical"), QKeySequence(), true);
    registry.addSeparator({"Image"});
    registry.add(command_ids::ImageCrop, {"Image", "Crop"}, QStringLiteral("Crop"),
                 QKeySequence(), true);
    leaf(registry, {"Image", "Trim…"}, QStringLiteral("Trim…"));
    leaf(registry, {"Image", "Reveal All"}, QStringLiteral("Reveal All"));
    registry.addSeparator({"Image"});
    leaf(registry, {"Image", "Variables", "Define…"}, QStringLiteral("Define…"));
    leaf(registry, {"Image", "Variables", "Data Sets…"}, QStringLiteral("Data Sets…"));
    leaf(registry, {"Image", "Apply Data Set…"}, QStringLiteral("Apply Data Set…"));
    registry.addSeparator({"Image"});
    leaf(registry, {"Image", "Trap…"}, QStringLiteral("Trap…"));
    leaf(registry, {"Image", "Apply Image…"}, QStringLiteral("Apply Image…"));
    leaf(registry, {"Image", "Calculations…"}, QStringLiteral("Calculations…"));

    // Layer
    leaf(registry, {"Layer", "New", "Layer"}, QStringLiteral("Layer"), QStringLiteral("Shift+Ctrl+N"));
    leaf(registry, {"Layer", "New", "Layer from Background…"},
         QStringLiteral("Layer from Background…"));
    leaf(registry, {"Layer", "New", "Group…"}, QStringLiteral("Group…"));
    leaf(registry, {"Layer", "New", "Group from Layers…"}, QStringLiteral("Group from Layers…"));
    leaf(registry, {"Layer", "New", "Layer via Copy"}, QStringLiteral("Layer via Copy"),
         QStringLiteral("Ctrl+J"));
    leaf(registry, {"Layer", "New", "Layer via Cut"}, QStringLiteral("Layer via Cut"),
         QStringLiteral("Shift+Ctrl+J"));
    leaf(registry, {"Layer", "New", "Background From Layer"},
         QStringLiteral("Background From Layer"));
    registry.addSeparator({"Layer"});
    leaf(registry, {"Layer", "Duplicate Layer…"}, QStringLiteral("Duplicate Layer…"));
    leaf(registry, {"Layer", "Delete Layer"}, QStringLiteral("Delete Layer"));
    leaf(registry, {"Layer", "Delete Hidden Layers"}, QStringLiteral("Delete Hidden Layers"));
    registry.addSeparator({"Layer"});
    leaf(registry, {"Layer", "Layer Style", "Blending Options…"},
         QStringLiteral("Blending Options…"));
    leaf(registry, {"Layer", "Layer Style", "Drop Shadow…"}, QStringLiteral("Drop Shadow…"));
    leaf(registry, {"Layer", "Layer Style", "Inner Shadow…"}, QStringLiteral("Inner Shadow…"));
    leaf(registry, {"Layer", "Layer Style", "Outer Glow…"}, QStringLiteral("Outer Glow…"));
    leaf(registry, {"Layer", "Layer Style", "Inner Glow…"}, QStringLiteral("Inner Glow…"));
    leaf(registry, {"Layer", "Layer Style", "Bevel & Emboss…"}, QStringLiteral("Bevel & Emboss…"));
    leaf(registry, {"Layer", "Layer Style", "Satin…"}, QStringLiteral("Satin…"));
    leaf(registry, {"Layer", "Layer Style", "Color Overlay…"}, QStringLiteral("Color Overlay…"));
    leaf(registry, {"Layer", "Layer Style", "Gradient Overlay…"},
         QStringLiteral("Gradient Overlay…"));
    leaf(registry, {"Layer", "Layer Style", "Pattern Overlay…"}, QStringLiteral("Pattern Overlay…"));
    leaf(registry, {"Layer", "Layer Style", "Stroke…"}, QStringLiteral("Stroke…"));
    registry.addSeparator({"Layer", "Layer Style"});
    leaf(registry, {"Layer", "Layer Style", "Copy Layer Style"}, QStringLiteral("Copy Layer Style"));
    leaf(registry, {"Layer", "Layer Style", "Paste Layer Style"},
         QStringLiteral("Paste Layer Style"));
    leaf(registry, {"Layer", "Layer Style", "Clear Layer Style"},
         QStringLiteral("Clear Layer Style"));
    leaf(registry, {"Layer", "Layer Style", "Global Light…"}, QStringLiteral("Global Light…"));
    leaf(registry, {"Layer", "Layer Style", "Create Layers"}, QStringLiteral("Create Layers"));
    leaf(registry, {"Layer", "Layer Style", "Scale Effects"}, QStringLiteral("Scale Effects"));
    leaf(registry, {"Layer", "Layer Style", "Hide All Effects"}, QStringLiteral("Hide All Effects"));
    leaf(registry, {"Layer", "Layer Style", "Show All Effects"}, QStringLiteral("Show All Effects"));
    leaf(registry, {"Layer", "Smart Filter", "Disable Filter Mask"},
         QStringLiteral("Disable Filter Mask"));
    leaf(registry, {"Layer", "Smart Filter", "Delete Filter Mask"},
         QStringLiteral("Delete Filter Mask"));
    leaf(registry, {"Layer", "New Fill Layer", "Solid Color…"}, QStringLiteral("Solid Color…"));
    leaf(registry, {"Layer", "New Fill Layer", "Gradient…"}, QStringLiteral("Gradient…"));
    leaf(registry, {"Layer", "New Fill Layer", "Pattern…"}, QStringLiteral("Pattern…"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Brightness/Contrast"},
         QStringLiteral("Brightness/Contrast"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Levels"}, QStringLiteral("Levels"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Curves"}, QStringLiteral("Curves"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Exposure"}, QStringLiteral("Exposure"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Vibrance"}, QStringLiteral("Vibrance"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Hue/Saturation"},
         QStringLiteral("Hue/Saturation"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Color Balance"},
         QStringLiteral("Color Balance"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Black & White"},
         QStringLiteral("Black & White"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Photo Filter"}, QStringLiteral("Photo Filter"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Channel Mixer"},
         QStringLiteral("Channel Mixer"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Color Lookup"}, QStringLiteral("Color Lookup"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Invert"}, QStringLiteral("Invert"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Posterize"}, QStringLiteral("Posterize"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Threshold"}, QStringLiteral("Threshold"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Gradient Map"}, QStringLiteral("Gradient Map"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Selective Color"},
         QStringLiteral("Selective Color"));
    leaf(registry, {"Layer", "New Adjustment Layer", "Shadows/Highlights"},
         QStringLiteral("Shadows/Highlights"));
    leaf(registry, {"Layer", "New Adjustment Layer", "HDR Toning"}, QStringLiteral("HDR Toning"));
    leaf(registry, {"Layer", "Layer Content Options…"}, QStringLiteral("Layer Content Options…"));
    leaf(registry, {"Layer", "Layer Mask", "Reveal All"}, QStringLiteral("Reveal All"));
    leaf(registry, {"Layer", "Layer Mask", "Hide All"}, QStringLiteral("Hide All"));
    leaf(registry, {"Layer", "Layer Mask", "Reveal Selection"}, QStringLiteral("Reveal Selection"));
    leaf(registry, {"Layer", "Layer Mask", "Hide Selection"}, QStringLiteral("Hide Selection"));
    leaf(registry, {"Layer", "Layer Mask", "From Transparency"}, QStringLiteral("From Transparency"));
    leaf(registry, {"Layer", "Layer Mask", "Delete"}, QStringLiteral("Delete"));
    leaf(registry, {"Layer", "Layer Mask", "Apply"}, QStringLiteral("Apply"));
    leaf(registry, {"Layer", "Layer Mask", "Enable"}, QStringLiteral("Enable"));
    leaf(registry, {"Layer", "Layer Mask", "Disable"}, QStringLiteral("Disable"));
    leaf(registry, {"Layer", "Layer Mask", "Link"}, QStringLiteral("Link"));
    leaf(registry, {"Layer", "Layer Mask", "Unlink"}, QStringLiteral("Unlink"));
    leaf(registry, {"Layer", "Vector Mask", "Reveal All"}, QStringLiteral("Reveal All"));
    leaf(registry, {"Layer", "Vector Mask", "Hide All"}, QStringLiteral("Hide All"));
    leaf(registry, {"Layer", "Vector Mask", "Current Path"}, QStringLiteral("Current Path"));
    leaf(registry, {"Layer", "Vector Mask", "Delete"}, QStringLiteral("Delete"));
    leaf(registry, {"Layer", "Vector Mask", "Enable"}, QStringLiteral("Enable"));
    leaf(registry, {"Layer", "Vector Mask", "Disable"}, QStringLiteral("Disable"));
    leaf(registry, {"Layer", "Vector Mask", "Link"}, QStringLiteral("Link"));
    leaf(registry, {"Layer", "Vector Mask", "Unlink"}, QStringLiteral("Unlink"));
    leaf(registry, {"Layer", "Create Clipping Mask"}, QStringLiteral("Create Clipping Mask"),
         QStringLiteral("Ctrl+Alt+G"));
    leaf(registry, {"Layer", "Release Clipping Mask"}, QStringLiteral("Release Clipping Mask"));
    leaf(registry, {"Layer", "Smart Objects", "Convert to Smart Object"},
         QStringLiteral("Convert to Smart Object"));
    leaf(registry, {"Layer", "Smart Objects", "New Smart Object via Copy"},
         QStringLiteral("New Smart Object via Copy"));
    leaf(registry, {"Layer", "Smart Objects", "Edit Contents"}, QStringLiteral("Edit Contents"));
    leaf(registry, {"Layer", "Smart Objects", "Export Contents…"},
         QStringLiteral("Export Contents…"));
    leaf(registry, {"Layer", "Smart Objects", "Replace Contents…"},
         QStringLiteral("Replace Contents…"));
    leaf(registry, {"Layer", "Smart Objects", "Rasterize"}, QStringLiteral("Rasterize"));
    leaf(registry, {"Layer", "Smart Objects", "Stack Mode"}, QStringLiteral("Stack Mode"));
    leaf(registry, {"Layer", "Video Layers", "New Blank Video Layer"},
         QStringLiteral("New Blank Video Layer"));
    leaf(registry, {"Layer", "Video Layers", "New Video Layer From File"},
         QStringLiteral("New Video Layer From File"));
    leaf(registry, {"Layer", "Video Layers", "Replace Footage"}, QStringLiteral("Replace Footage"));
    leaf(registry, {"Layer", "Video Layers", "Interpret Footage"},
         QStringLiteral("Interpret Footage"));
    leaf(registry, {"Layer", "Rasterize", "Type"}, QStringLiteral("Type"));
    leaf(registry, {"Layer", "Rasterize", "Shape"}, QStringLiteral("Shape"));
    leaf(registry, {"Layer", "Rasterize", "Fill Content"}, QStringLiteral("Fill Content"));
    leaf(registry, {"Layer", "Rasterize", "Layer"}, QStringLiteral("Layer"));
    leaf(registry, {"Layer", "Rasterize", "Layer Style"}, QStringLiteral("Layer Style"));
    leaf(registry, {"Layer", "Rasterize", "Video"}, QStringLiteral("Video"));
    leaf(registry, {"Layer", "Rasterize", "3D"}, QStringLiteral("3D"));
    leaf(registry, {"Layer", "New Layer-based Slice"}, QStringLiteral("New Layer-based Slice"));
    leaf(registry, {"Layer", "Group Layers"}, QStringLiteral("Group Layers"), QStringLiteral("Ctrl+G"));
    leaf(registry, {"Layer", "Ungroup Layers"}, QStringLiteral("Ungroup Layers"),
         QStringLiteral("Shift+Ctrl+G"));
    leaf(registry, {"Layer", "Hide Layers"}, QStringLiteral("Hide Layers"));
    leaf(registry, {"Layer", "Arrange", "Bring to Front"}, QStringLiteral("Bring to Front"),
         QStringLiteral("Shift+Ctrl+]"));
    leaf(registry, {"Layer", "Arrange", "Bring Forward"}, QStringLiteral("Bring Forward"),
         QStringLiteral("Ctrl+]"));
    leaf(registry, {"Layer", "Arrange", "Send Backward"}, QStringLiteral("Send Backward"),
         QStringLiteral("Ctrl+["));
    leaf(registry, {"Layer", "Arrange", "Send to Back"}, QStringLiteral("Send to Back"),
         QStringLiteral("Shift+Ctrl+["));
    leaf(registry, {"Layer", "Align", "Top"}, QStringLiteral("Top"));
    leaf(registry, {"Layer", "Align", "Vertical Center"}, QStringLiteral("Vertical Center"));
    leaf(registry, {"Layer", "Align", "Bottom"}, QStringLiteral("Bottom"));
    leaf(registry, {"Layer", "Align", "Left"}, QStringLiteral("Left"));
    leaf(registry, {"Layer", "Align", "Horizontal Center"}, QStringLiteral("Horizontal Center"));
    leaf(registry, {"Layer", "Align", "Right"}, QStringLiteral("Right"));
    leaf(registry, {"Layer", "Align Layers To Selection", "Top"}, QStringLiteral("Top"));
    leaf(registry, {"Layer", "Align Layers To Selection", "Vertical Center"},
         QStringLiteral("Vertical Center"));
    leaf(registry, {"Layer", "Align Layers To Selection", "Bottom"}, QStringLiteral("Bottom"));
    leaf(registry, {"Layer", "Align Layers To Selection", "Left"}, QStringLiteral("Left"));
    leaf(registry, {"Layer", "Align Layers To Selection", "Horizontal Center"},
         QStringLiteral("Horizontal Center"));
    leaf(registry, {"Layer", "Align Layers To Selection", "Right"}, QStringLiteral("Right"));
    leaf(registry, {"Layer", "Distribute", "Top"}, QStringLiteral("Top"));
    leaf(registry, {"Layer", "Distribute", "Vertical Center"}, QStringLiteral("Vertical Center"));
    leaf(registry, {"Layer", "Distribute", "Bottom"}, QStringLiteral("Bottom"));
    leaf(registry, {"Layer", "Distribute", "Left"}, QStringLiteral("Left"));
    leaf(registry, {"Layer", "Distribute", "Horizontal Center"}, QStringLiteral("Horizontal Center"));
    leaf(registry, {"Layer", "Distribute", "Right"}, QStringLiteral("Right"));
    leaf(registry, {"Layer", "Lock All Layers In Group…"},
         QStringLiteral("Lock All Layers In Group…"));
    leaf(registry, {"Layer", "Merge Layers"}, QStringLiteral("Merge Layers"), QStringLiteral("Ctrl+E"));
    leaf(registry, {"Layer", "Merge Visible"}, QStringLiteral("Merge Visible"),
         QStringLiteral("Shift+Ctrl+E"));
    leaf(registry, {"Layer", "Flatten Image"}, QStringLiteral("Flatten Image"));
    leaf(registry, {"Layer", "Matting", "Defringe…"}, QStringLiteral("Defringe…"));
    leaf(registry, {"Layer", "Matting", "Remove Black Matte"}, QStringLiteral("Remove Black Matte"));
    leaf(registry, {"Layer", "Matting", "Remove White Matte"}, QStringLiteral("Remove White Matte"));

    // Type
    leaf(registry, {"Type", "Panels", "Character"}, QStringLiteral("Character"));
    leaf(registry, {"Type", "Panels", "Paragraph"}, QStringLiteral("Paragraph"));
    leaf(registry, {"Type", "Panels", "Character Styles"}, QStringLiteral("Character Styles"));
    leaf(registry, {"Type", "Panels", "Paragraph Styles"}, QStringLiteral("Paragraph Styles"));
    leaf(registry, {"Type", "Anti-Alias", "None"}, QStringLiteral("None"));
    leaf(registry, {"Type", "Anti-Alias", "Sharp"}, QStringLiteral("Sharp"));
    leaf(registry, {"Type", "Anti-Alias", "Crisp"}, QStringLiteral("Crisp"));
    leaf(registry, {"Type", "Anti-Alias", "Strong"}, QStringLiteral("Strong"));
    leaf(registry, {"Type", "Anti-Alias", "Smooth"}, QStringLiteral("Smooth"));
    leaf(registry, {"Type", "Orientation", "Horizontal"}, QStringLiteral("Horizontal"));
    leaf(registry, {"Type", "Orientation", "Vertical"}, QStringLiteral("Vertical"));
    leaf(registry, {"Type", "Convert To Point Text"}, QStringLiteral("Convert To Point Text"));
    leaf(registry, {"Type", "Convert To Paragraph Text"},
         QStringLiteral("Convert To Paragraph Text"));
    leaf(registry, {"Type", "Warp Text…"}, QStringLiteral("Warp Text…"));
    leaf(registry, {"Type", "Rasterize Type Layer"}, QStringLiteral("Rasterize Type Layer"));
    leaf(registry, {"Type", "Create Work Path"}, QStringLiteral("Create Work Path"));
    leaf(registry, {"Type", "Convert to Shape"}, QStringLiteral("Convert to Shape"));
    leaf(registry, {"Type", "Font Preview Size", "None"}, QStringLiteral("None"));
    leaf(registry, {"Type", "Font Preview Size", "Small"}, QStringLiteral("Small"));
    leaf(registry, {"Type", "Font Preview Size", "Medium"}, QStringLiteral("Medium"));
    leaf(registry, {"Type", "Font Preview Size", "Large"}, QStringLiteral("Large"));
    leaf(registry, {"Type", "Font Preview Size", "Extra Large"}, QStringLiteral("Extra Large"));
    leaf(registry, {"Type", "Font Preview Size", "Huge"}, QStringLiteral("Huge"));
    leaf(registry, {"Type", "Language Options", "Middle Eastern features"},
         QStringLiteral("Middle Eastern features"));
    leaf(registry, {"Type", "Language Options", "East Asian features"},
         QStringLiteral("East Asian features"));
    leaf(registry, {"Type", "Update All Text Layers"}, QStringLiteral("Update All Text Layers"));
    leaf(registry, {"Type", "Check Spelling…"}, QStringLiteral("Check Spelling…"));
    leaf(registry, {"Type", "Find And Replace Text…"}, QStringLiteral("Find And Replace Text…"));
    leaf(registry, {"Type", "Paste Lorem Ipsum"}, QStringLiteral("Paste Lorem Ipsum"));
    leaf(registry, {"Type", "Load Default Type Styles"}, QStringLiteral("Load Default Type Styles"));
    leaf(registry, {"Type", "Save Default Type Styles"}, QStringLiteral("Save Default Type Styles"));
    leaf(registry, {"Type", "Extrude to 3D"}, QStringLiteral("Extrude to 3D"));

    // Select
    registry.add(command_ids::SelectAll, {"Select", "All"}, QStringLiteral("All"),
                 QKeySequence(QStringLiteral("Ctrl+A")), true);
    registry.add(command_ids::SelectDeselect, {"Select", "Deselect"}, QStringLiteral("Deselect"),
                 QKeySequence(QStringLiteral("Ctrl+D")), true);
    registry.addSeparator({"Select"});
    leaf(registry, {"Select", "Reselect"}, QStringLiteral("Reselect"), QStringLiteral("Shift+Ctrl+D"));
    leaf(registry, {"Select", "Inverse"}, QStringLiteral("Inverse"), QStringLiteral("Shift+Ctrl+I"));
    registry.addSeparator({"Select"});
    leaf(registry, {"Select", "All Layers"}, QStringLiteral("All Layers"), QStringLiteral("Ctrl+Alt+A"));
    leaf(registry, {"Select", "Deselect Layers"}, QStringLiteral("Deselect Layers"));
    leaf(registry, {"Select", "Similar Layers"}, QStringLiteral("Similar Layers"));
    registry.addSeparator({"Select"});
    leaf(registry, {"Select", "Color Range…"}, QStringLiteral("Color Range…"));
    leaf(registry, {"Select", "Refine Edge…"}, QStringLiteral("Refine Edge…"),
         QStringLiteral("Ctrl+Alt+R"));
    registry.addSeparator({"Select"});
    leaf(registry, {"Select", "Modify", "Border…"}, QStringLiteral("Border…"));
    leaf(registry, {"Select", "Modify", "Smooth…"}, QStringLiteral("Smooth…"));
    leaf(registry, {"Select", "Modify", "Expand…"}, QStringLiteral("Expand…"));
    leaf(registry, {"Select", "Modify", "Contract…"}, QStringLiteral("Contract…"));
    leaf(registry, {"Select", "Modify", "Feather…"}, QStringLiteral("Feather…"),
         QStringLiteral("Shift+F6"));
    registry.addSeparator({"Select"});
    leaf(registry, {"Select", "Grow"}, QStringLiteral("Grow"));
    leaf(registry, {"Select", "Similar"}, QStringLiteral("Similar"));
    registry.addSeparator({"Select"});
    leaf(registry, {"Select", "Transform Selection"}, QStringLiteral("Transform Selection"));
    leaf(registry, {"Select", "Save Selection…"}, QStringLiteral("Save Selection…"));
    leaf(registry, {"Select", "Load Selection…"}, QStringLiteral("Load Selection…"));

    // Filter
    leaf(registry, {"Filter", "Last Filter"}, QStringLiteral("Last Filter"), QStringLiteral("Ctrl+F"));
    leaf(registry, {"Filter", "Convert for Smart Filters"},
         QStringLiteral("Convert for Smart Filters"));
    registry.addSeparator({"Filter"});
    leaf(registry, {"Filter", "Filter Gallery…"}, QStringLiteral("Filter Gallery…"));
    leaf(registry, {"Filter", "Adaptive Wide Angle…"}, QStringLiteral("Adaptive Wide Angle…"));
    leaf(registry, {"Filter", "Camera Raw Filter…"}, QStringLiteral("Camera Raw Filter…"));
    leaf(registry, {"Filter", "Lens Correction…"}, QStringLiteral("Lens Correction…"));
    leaf(registry, {"Filter", "Liquify…"}, QStringLiteral("Liquify…"), QStringLiteral("Shift+Ctrl+X"));
    leaf(registry, {"Filter", "Oil Paint…"}, QStringLiteral("Oil Paint…"));
    leaf(registry, {"Filter", "Vanishing Point…"}, QStringLiteral("Vanishing Point…"),
         QStringLiteral("Alt+Ctrl+V"));
    registry.addSeparator({"Filter"});
    leaf(registry, {"Filter", "Blur", "Field Blur"}, QStringLiteral("Field Blur"));
    leaf(registry, {"Filter", "Blur", "Iris Blur"}, QStringLiteral("Iris Blur"));
    leaf(registry, {"Filter", "Blur", "Tilt-Shift"}, QStringLiteral("Tilt-Shift"));
    leaf(registry, {"Filter", "Blur", "Gaussian Blur"}, QStringLiteral("Gaussian Blur"));
    leaf(registry, {"Filter", "Blur", "Motion Blur"}, QStringLiteral("Motion Blur"));
    leaf(registry, {"Filter", "Blur", "Radial Blur"}, QStringLiteral("Radial Blur"));
    leaf(registry, {"Filter", "Blur", "Box Blur"}, QStringLiteral("Box Blur"));
    leaf(registry, {"Filter", "Blur", "Surface Blur"}, QStringLiteral("Surface Blur"));
    leaf(registry, {"Filter", "Blur", "Lens Blur"}, QStringLiteral("Lens Blur"));
    leaf(registry, {"Filter", "Blur", "Smart Blur"}, QStringLiteral("Smart Blur"));
    leaf(registry, {"Filter", "Blur", "Average"}, QStringLiteral("Average"));
    leaf(registry, {"Filter", "Brush Strokes", "Accented Edges"}, QStringLiteral("Accented Edges"));
    leaf(registry, {"Filter", "Brush Strokes", "Angled Strokes"}, QStringLiteral("Angled Strokes"));
    leaf(registry, {"Filter", "Brush Strokes", "Crosshatch"}, QStringLiteral("Crosshatch"));
    leaf(registry, {"Filter", "Brush Strokes", "Dark Strokes"}, QStringLiteral("Dark Strokes"));
    leaf(registry, {"Filter", "Brush Strokes", "Ink Outlines"}, QStringLiteral("Ink Outlines"));
    leaf(registry, {"Filter", "Brush Strokes", "Spatter"}, QStringLiteral("Spatter"));
    leaf(registry, {"Filter", "Brush Strokes", "Sprayed Strokes"}, QStringLiteral("Sprayed Strokes"));
    leaf(registry, {"Filter", "Brush Strokes", "Sumi-e"}, QStringLiteral("Sumi-e"));
    leaf(registry, {"Filter", "Distort", "Displace"}, QStringLiteral("Displace"));
    leaf(registry, {"Filter", "Distort", "Glass"}, QStringLiteral("Glass"));
    leaf(registry, {"Filter", "Distort", "Ocean Ripple"}, QStringLiteral("Ocean Ripple"));
    leaf(registry, {"Filter", "Distort", "Diffuse Glow"}, QStringLiteral("Diffuse Glow"));
    leaf(registry, {"Filter", "Distort", "Pinch"}, QStringLiteral("Pinch"));
    leaf(registry, {"Filter", "Distort", "Polar Coordinates"}, QStringLiteral("Polar Coordinates"));
    leaf(registry, {"Filter", "Distort", "Ripple"}, QStringLiteral("Ripple"));
    leaf(registry, {"Filter", "Distort", "Shear"}, QStringLiteral("Shear"));
    leaf(registry, {"Filter", "Distort", "Spherize"}, QStringLiteral("Spherize"));
    leaf(registry, {"Filter", "Distort", "Twirl"}, QStringLiteral("Twirl"));
    leaf(registry, {"Filter", "Distort", "Wave"}, QStringLiteral("Wave"));
    leaf(registry, {"Filter", "Distort", "ZigZag"}, QStringLiteral("ZigZag"));
    leaf(registry, {"Filter", "Noise", "Add Noise"}, QStringLiteral("Add Noise"));
    leaf(registry, {"Filter", "Noise", "Despeckle"}, QStringLiteral("Despeckle"));
    leaf(registry, {"Filter", "Noise", "Dust & Scratches"}, QStringLiteral("Dust & Scratches"));
    leaf(registry, {"Filter", "Noise", "Median"}, QStringLiteral("Median"));
    leaf(registry, {"Filter", "Noise", "Reduce Noise"}, QStringLiteral("Reduce Noise"));
    leaf(registry, {"Filter", "Pixelate", "Color Halftone"}, QStringLiteral("Color Halftone"));
    leaf(registry, {"Filter", "Pixelate", "Crystallize"}, QStringLiteral("Crystallize"));
    leaf(registry, {"Filter", "Pixelate", "Facet"}, QStringLiteral("Facet"));
    leaf(registry, {"Filter", "Pixelate", "Fragment"}, QStringLiteral("Fragment"));
    leaf(registry, {"Filter", "Pixelate", "Mezzotint"}, QStringLiteral("Mezzotint"));
    leaf(registry, {"Filter", "Pixelate", "Mosaic"}, QStringLiteral("Mosaic"));
    leaf(registry, {"Filter", "Pixelate", "Pointillize"}, QStringLiteral("Pointillize"));
    leaf(registry, {"Filter", "Render", "Clouds"}, QStringLiteral("Clouds"));
    leaf(registry, {"Filter", "Render", "Difference Clouds"}, QStringLiteral("Difference Clouds"));
    leaf(registry, {"Filter", "Render", "Fibers"}, QStringLiteral("Fibers"));
    leaf(registry, {"Filter", "Render", "Lens Flare"}, QStringLiteral("Lens Flare"));
    leaf(registry, {"Filter", "Render", "Lighting Effects"}, QStringLiteral("Lighting Effects"));
    leaf(registry, {"Filter", "Sharpen", "Smart Sharpen"}, QStringLiteral("Smart Sharpen"));
    leaf(registry, {"Filter", "Sharpen", "Unsharp Mask"}, QStringLiteral("Unsharp Mask"));
    leaf(registry, {"Filter", "Sharpen", "Sharpen"}, QStringLiteral("Sharpen"));
    leaf(registry, {"Filter", "Sharpen", "Sharpen Edges"}, QStringLiteral("Sharpen Edges"));
    leaf(registry, {"Filter", "Sharpen", "Sharpen More"}, QStringLiteral("Sharpen More"));
    leaf(registry, {"Filter", "Sketch", "Bas Relief"}, QStringLiteral("Bas Relief"));
    leaf(registry, {"Filter", "Sketch", "Chalk & Charcoal"}, QStringLiteral("Chalk & Charcoal"));
    leaf(registry, {"Filter", "Sketch", "Charcoal"}, QStringLiteral("Charcoal"));
    leaf(registry, {"Filter", "Sketch", "Chrome"}, QStringLiteral("Chrome"));
    leaf(registry, {"Filter", "Sketch", "Conté Crayon"}, QStringLiteral("Conté Crayon"));
    leaf(registry, {"Filter", "Sketch", "Graphic Pen"}, QStringLiteral("Graphic Pen"));
    leaf(registry, {"Filter", "Sketch", "Halftone Pattern"}, QStringLiteral("Halftone Pattern"));
    leaf(registry, {"Filter", "Sketch", "Note Paper"}, QStringLiteral("Note Paper"));
    leaf(registry, {"Filter", "Sketch", "Photocopy"}, QStringLiteral("Photocopy"));
    leaf(registry, {"Filter", "Sketch", "Plaster"}, QStringLiteral("Plaster"));
    leaf(registry, {"Filter", "Sketch", "Reticulation"}, QStringLiteral("Reticulation"));
    leaf(registry, {"Filter", "Sketch", "Stamp"}, QStringLiteral("Stamp"));
    leaf(registry, {"Filter", "Sketch", "Torn Edges"}, QStringLiteral("Torn Edges"));
    leaf(registry, {"Filter", "Sketch", "Water Paper"}, QStringLiteral("Water Paper"));
    leaf(registry, {"Filter", "Stylize", "Diffuse"}, QStringLiteral("Diffuse"));
    leaf(registry, {"Filter", "Stylize", "Emboss"}, QStringLiteral("Emboss"));
    leaf(registry, {"Filter", "Stylize", "Extrude"}, QStringLiteral("Extrude"));
    leaf(registry, {"Filter", "Stylize", "Find Edges"}, QStringLiteral("Find Edges"));
    leaf(registry, {"Filter", "Stylize", "Glowing Edges"}, QStringLiteral("Glowing Edges"));
    leaf(registry, {"Filter", "Stylize", "Solarize"}, QStringLiteral("Solarize"));
    leaf(registry, {"Filter", "Stylize", "Tiles"}, QStringLiteral("Tiles"));
    leaf(registry, {"Filter", "Stylize", "Trace Contour"}, QStringLiteral("Trace Contour"));
    leaf(registry, {"Filter", "Stylize", "Wind"}, QStringLiteral("Wind"));
    leaf(registry, {"Filter", "Texture", "Craquelure"}, QStringLiteral("Craquelure"));
    leaf(registry, {"Filter", "Texture", "Grain"}, QStringLiteral("Grain"));
    leaf(registry, {"Filter", "Texture", "Mosaic Tiles"}, QStringLiteral("Mosaic Tiles"));
    leaf(registry, {"Filter", "Texture", "Patchwork"}, QStringLiteral("Patchwork"));
    leaf(registry, {"Filter", "Texture", "Stained Glass"}, QStringLiteral("Stained Glass"));
    leaf(registry, {"Filter", "Texture", "Texturizer"}, QStringLiteral("Texturizer"));
    leaf(registry, {"Filter", "Video", "De-Interlace"}, QStringLiteral("De-Interlace"));
    leaf(registry, {"Filter", "Video", "NTSC Colors"}, QStringLiteral("NTSC Colors"));
    leaf(registry, {"Filter", "Other", "Custom"}, QStringLiteral("Custom"));
    leaf(registry, {"Filter", "Other", "High Pass"}, QStringLiteral("High Pass"));
    leaf(registry, {"Filter", "Other", "Maximum"}, QStringLiteral("Maximum"));
    leaf(registry, {"Filter", "Other", "Minimum"}, QStringLiteral("Minimum"));
    leaf(registry, {"Filter", "Other", "Offset"}, QStringLiteral("Offset"));
    leaf(registry, {"Filter", "Digimarc", "Embed Watermark"}, QStringLiteral("Embed Watermark"));
    leaf(registry, {"Filter", "Digimarc", "Read Watermark"}, QStringLiteral("Read Watermark"));
    registry.addSeparator({"Filter"});
    leaf(registry, {"Filter", "Extract…"}, QStringLiteral("Extract…"));
    leaf(registry, {"Filter", "Pattern Maker…"}, QStringLiteral("Pattern Maker…"));
    registry.addSeparator({"Filter"});
    leaf(registry, {"Filter", "Browse Filters Online…"}, QStringLiteral("Browse Filters Online…"));

    // View
    leaf(registry, {"View", "Proof Setup", "Working CMYK"}, QStringLiteral("Working CMYK"));
    leaf(registry, {"View", "Proof Setup", "Working Gray"}, QStringLiteral("Working Gray"));
    leaf(registry, {"View", "Proof Setup", "Custom"}, QStringLiteral("Custom"));
    leaf(registry, {"View", "Proof Setup", "Color Blindness", "Protanopia-type"},
         QStringLiteral("Protanopia-type"));
    leaf(registry, {"View", "Proof Setup", "Color Blindness", "Deuteranopia-type"},
         QStringLiteral("Deuteranopia-type"));
    leaf(registry, {"View", "Proof Colors"}, QStringLiteral("Proof Colors"), QStringLiteral("Ctrl+Y"));
    leaf(registry, {"View", "Gamut Warning"}, QStringLiteral("Gamut Warning"),
         QStringLiteral("Shift+Ctrl+Y"));
    registry.addSeparator({"View"});
    registry.add(command_ids::ViewZoomIn, {"View", "Zoom In"}, QStringLiteral("Zoom In"),
                 QKeySequence(QStringLiteral("Ctrl++")), true);
    registry.add(command_ids::ViewZoomOut, {"View", "Zoom Out"}, QStringLiteral("Zoom Out"),
                 QKeySequence(QStringLiteral("Ctrl+-")), true);
    registry.add(command_ids::ViewFitOnScreen, {"View", "Fit On Screen"},
                 QStringLiteral("Fit On Screen"), QKeySequence(QStringLiteral("Ctrl+0")), true);
    registry.add(command_ids::ViewActualPixels, {"View", "100%"}, QStringLiteral("100%"),
                 QKeySequence(QStringLiteral("Ctrl+1")), true);
    leaf(registry, {"View", "200%"}, QStringLiteral("200%"));
    leaf(registry, {"View", "Print Size"}, QStringLiteral("Print Size"));
    registry.addSeparator({"View"});
    registry.add(CommandSpec{command_ids::ViewScreenModeStandard,
                             {"View", "Screen Mode", "Standard Screen Mode"},
                             QStringLiteral("Standard Screen Mode"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::ViewScreenModeFullWithMenuBar,
                             {"View", "Screen Mode", "Full Screen Mode With Menu Bar"},
                             QStringLiteral("Full Screen Mode With Menu Bar"), QKeySequence(), true,
                             true});
    registry.add(CommandSpec{command_ids::ViewScreenModeFull,
                             {"View", "Screen Mode", "Full Screen Mode"},
                             QStringLiteral("Full Screen Mode"), QKeySequence(), true, true});
    registry.addSeparator({"View"});
    leaf(registry, {"View", "Rulers"}, QStringLiteral("Rulers"), QStringLiteral("Ctrl+R"));
    leaf(registry, {"View", "Snap"}, QStringLiteral("Snap"), QStringLiteral("Shift+Ctrl+;"));
    leaf(registry, {"View", "Snap To", "Guides"}, QStringLiteral("Guides"));
    leaf(registry, {"View", "Snap To", "Grid"}, QStringLiteral("Grid"));
    leaf(registry, {"View", "Snap To", "Layers"}, QStringLiteral("Layers"));
    leaf(registry, {"View", "Snap To", "Slices"}, QStringLiteral("Slices"));
    leaf(registry, {"View", "Snap To", "Document Bounds"}, QStringLiteral("Document Bounds"));
    leaf(registry, {"View", "Snap To", "All"}, QStringLiteral("All"));
    leaf(registry, {"View", "Snap To", "None"}, QStringLiteral("None"));
    leaf(registry, {"View", "Lock Guides"}, QStringLiteral("Lock Guides"), QStringLiteral("Alt+Ctrl+;"));
    leaf(registry, {"View", "Clear Guides"}, QStringLiteral("Clear Guides"));
    leaf(registry, {"View", "New Guide…"}, QStringLiteral("New Guide…"));
    registry.addSeparator({"View"});
    leaf(registry, {"View", "Lock Slices"}, QStringLiteral("Lock Slices"));
    registry.addSeparator({"View"});
    leaf(registry, {"View", "Show", "Selection Edges"}, QStringLiteral("Selection Edges"));
    leaf(registry, {"View", "Show", "Target Path"}, QStringLiteral("Target Path"));
    leaf(registry, {"View", "Show", "Layer Edges"}, QStringLiteral("Layer Edges"));
    leaf(registry, {"View", "Show", "3D Axis"}, QStringLiteral("3D Axis"));
    leaf(registry, {"View", "Show", "Guides"}, QStringLiteral("Guides"));
    leaf(registry, {"View", "Show", "Grid"}, QStringLiteral("Grid"));
    leaf(registry, {"View", "Show", "Count"}, QStringLiteral("Count"));
    leaf(registry, {"View", "Show", "Slices"}, QStringLiteral("Slices"));
    leaf(registry, {"View", "Show", "Notes"}, QStringLiteral("Notes"));
    leaf(registry, {"View", "Show", "Pixel Grid"}, QStringLiteral("Pixel Grid"));
    leaf(registry, {"View", "Show", "All"}, QStringLiteral("All"));
    leaf(registry, {"View", "Show", "None"}, QStringLiteral("None"));
    leaf(registry, {"View", "Extras"}, QStringLiteral("Extras"), QStringLiteral("Ctrl+H"));
    registry.addSeparator({"View"});
    leaf(registry, {"View", "Align"}, QStringLiteral("Align"));
    leaf(registry, {"View", "Align To"}, QStringLiteral("Align To"));
    registry.addSeparator({"View"});
    registry.add(CommandSpec{command_ids::ViewOptions, {"View", "Options"},
                             QStringLiteral("Options"), QKeySequence(), true, true});

    // Window
    leaf(registry, {"Window", "Arrange", "Cascade"}, QStringLiteral("Cascade"));
    leaf(registry, {"Window", "Arrange", "Tile"}, QStringLiteral("Tile"));
    leaf(registry, {"Window", "Arrange", "Consolidate All to Tabs"},
         QStringLiteral("Consolidate All to Tabs"));
    leaf(registry, {"Window", "Arrange", "Float in Window"}, QStringLiteral("Float in Window"));
    leaf(registry, {"Window", "Arrange", "Float All in Windows"},
         QStringLiteral("Float All in Windows"));
    leaf(registry, {"Window", "Arrange", "Match Zoom"}, QStringLiteral("Match Zoom"));
    leaf(registry, {"Window", "Arrange", "Match Location"}, QStringLiteral("Match Location"));
    leaf(registry, {"Window", "Arrange", "Match All"}, QStringLiteral("Match All"));
    leaf(registry, {"Window", "Arrange", "New Window For [file]"},
         QStringLiteral("New Window For [file]"));
    leaf(registry, {"Window", "Workspace", "Essentials"}, QStringLiteral("Essentials"));
    leaf(registry, {"Window", "Workspace", "New in CS6"}, QStringLiteral("New in CS6"));
    leaf(registry, {"Window", "Workspace", "3D"}, QStringLiteral("3D"));
    leaf(registry, {"Window", "Workspace", "Motion"}, QStringLiteral("Motion"));
    leaf(registry, {"Window", "Workspace", "Painting"}, QStringLiteral("Painting"));
    leaf(registry, {"Window", "Workspace", "Photography"}, QStringLiteral("Photography"));
    leaf(registry, {"Window", "Workspace", "Typography"}, QStringLiteral("Typography"));
    leaf(registry, {"Window", "Workspace", "Advanced 3D"}, QStringLiteral("Advanced 3D"));
    leaf(registry, {"Window", "Workspace", "New Workspace…"}, QStringLiteral("New Workspace…"));
    leaf(registry, {"Window", "Workspace", "Delete Workspace…"}, QStringLiteral("Delete Workspace…"));
    leaf(registry, {"Window", "Workspace", "Reset [Workspace]"}, QStringLiteral("Reset [Workspace]"));
    leaf(registry, {"Window", "Workspace", "Keyboard Shortcuts & Menus…"},
         QStringLiteral("Keyboard Shortcuts & Menus…"));
    registry.addSeparator({"Window"});
    leaf(registry, {"Window", "Panels", "3D"}, QStringLiteral("3D"));
    registry.add(CommandSpec{command_ids::WindowPanelsActions, {"Window", "Panels", "Actions"},
                             QStringLiteral("Actions"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsAdjustments, {"Window", "Panels", "Adjustments"},
                             QStringLiteral("Adjustments"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Animation"}, QStringLiteral("Animation"));
    leaf(registry, {"Window", "Panels", "Brush"}, QStringLiteral("Brush"));
    leaf(registry, {"Window", "Panels", "Brush Presets"}, QStringLiteral("Brush Presets"));
    registry.add(CommandSpec{command_ids::WindowPanelsChannels, {"Window", "Panels", "Channels"},
                             QStringLiteral("Channels"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Character"}, QStringLiteral("Character"));
    leaf(registry, {"Window", "Panels", "Character Styles"}, QStringLiteral("Character Styles"));
    leaf(registry, {"Window", "Panels", "Clone Source"}, QStringLiteral("Clone Source"));
    registry.add(CommandSpec{command_ids::WindowPanelsColor, {"Window", "Panels", "Color"},
                             QStringLiteral("Color"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsGradients, {"Window", "Panels", "Gradients"},
                             QStringLiteral("Gradients"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsHistogram, {"Window", "Panels", "Histogram"},
                             QStringLiteral("Histogram"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsHistory, {"Window", "Panels", "History"},
                             QStringLiteral("History"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsInfo, {"Window", "Panels", "Info"},
                             QStringLiteral("Info"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Layer Comps"}, QStringLiteral("Layer Comps"));
    registry.add(CommandSpec{command_ids::WindowPanelsLayers, {"Window", "Panels", "Layers"},
                             QStringLiteral("Layers"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsLibraries, {"Window", "Panels", "Libraries"},
                             QStringLiteral("Libraries"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Measurement Log"}, QStringLiteral("Measurement Log"));
    registry.add(CommandSpec{command_ids::WindowPanelsNavigator, {"Window", "Panels", "Navigator"},
                             QStringLiteral("Navigator"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Notes"}, QStringLiteral("Notes"));
    leaf(registry, {"Window", "Panels", "Paragraph"}, QStringLiteral("Paragraph"));
    leaf(registry, {"Window", "Panels", "Paragraph Styles"}, QStringLiteral("Paragraph Styles"));
    registry.add(CommandSpec{command_ids::WindowPanelsPaths, {"Window", "Panels", "Paths"},
                             QStringLiteral("Paths"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsPatterns, {"Window", "Panels", "Patterns"},
                             QStringLiteral("Patterns"), QKeySequence(), true, true});
    registry.add(CommandSpec{command_ids::WindowPanelsProperties, {"Window", "Panels", "Properties"},
                             QStringLiteral("Properties"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Styles"}, QStringLiteral("Styles"));
    registry.add(CommandSpec{command_ids::WindowPanelsSwatches, {"Window", "Panels", "Swatches"},
                             QStringLiteral("Swatches"), QKeySequence(), true, true});
    leaf(registry, {"Window", "Panels", "Timeline"}, QStringLiteral("Timeline"));
    leaf(registry, {"Window", "Panels", "Tool Presets"}, QStringLiteral("Tool Presets"));
    registry.add(CommandSpec{command_ids::WindowPanelsTools, {"Window", "Panels", "Tools"},
                             QStringLiteral("Tools"), QKeySequence(), true, true});
    registry.addSeparator({"Window"});
    leaf(registry, {"Window", "Extensions", "Mini Bridge"}, QStringLiteral("Mini Bridge"));
    registry.addSeparator({"Window"});
    leaf(registry, {"Window", "3D"}, QStringLiteral("3D"));

    // Help
    leaf(registry, {"Help", "Photoshop Help"}, QStringLiteral("Photoshop Help"), QStringLiteral("F1"));
    leaf(registry, {"Help", "Photoshop Support Center"},
         QStringLiteral("Photoshop Support Center"));
    leaf(registry, {"Help", "Full Product Family Help"}, QStringLiteral("Full Product Family Help"));
    leaf(registry, {"Help", "Adobe Product Improvement Program"},
         QStringLiteral("Adobe Product Improvement Program"));
    registry.addSeparator({"Help"});
    leaf(registry, {"Help", "About Plug-in"}, QStringLiteral("About Plug-in"));
    leaf(registry, {"Help", "System Info"}, QStringLiteral("System Info"));
    registry.addSeparator({"Help"});
    registry.add(command_ids::HelpAbout, {"Help", "About Photoshop"},
                 QStringLiteral("About Photoshop"), QKeySequence(), true);
}

} // namespace pictura
