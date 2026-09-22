//! Metadata templates: export the managed XMP set and apply it with a merge
//! mode.
//!
//! Apply is a thin selection layer over the existing writers: the six shared
//! IPTC-Core fields go through [`set_file_info_fields`] (which truncates to the
//! IIM length symmetrically and gates IIM on the XMP result), the XMP-only
//! fields through [`set_xmp_values`], so unknown namespaces, EXIF, and every
//! other resource survive because both writers patch in place.

use pictura_core::Document;

use crate::xmp::{to_xmp_packet, XmpField, XmpProperties};

use super::{set_file_info_fields, set_xmp_values, truncate_to_iim, xmp_properties};

/// How an applied template merges with the document's current metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMode {
    /// Fill a managed field only when the document's value is empty/absent.
    Append,
    /// Set every managed field, clearing the ones the template omits.
    Replace,
    /// Overwrite only the fields the template defines; clear nothing.
    KeepOriginalReplaceMatching,
}

const MANAGED: [XmpField; 9] = [
    XmpField::Title,
    XmpField::Creator,
    XmpField::Description,
    XmpField::Subject,
    XmpField::Rights,
    XmpField::Credit,
    XmpField::Source,
    XmpField::Headline,
    XmpField::Marked,
];

/// The document's managed XMP properties as a standalone template packet.
pub fn export_template(document: &Document) -> Vec<u8> {
    to_xmp_packet(&xmp_properties(document)).into_bytes()
}

/// Apply a template to a document in place with the given merge mode. Returns
/// whether either channel changed.
pub fn apply_template(document: &mut Document, template: &XmpProperties, mode: MergeMode) -> bool {
    let updates = select_updates(&xmp_properties(document), template, mode);
    if updates.is_empty() {
        return false;
    }

    // The shared fields carry one value to IIM; the list fields always rewrite
    // their full XMP list (one item or empty must be able to shrink/clear an
    // existing longer list).
    let mut shared: Vec<(u8, u8, String)> = Vec::new();
    let mut xmp_values: Vec<(XmpField, Vec<String>)> = Vec::new();
    for (field, values) in &updates {
        let shared_mapping = iim_for(*field);
        if let Some((record, dataset)) = shared_mapping {
            shared.push((record, dataset, values.first().cloned().unwrap_or_default()));
        }
        if shared_mapping.is_none() || is_list(*field) {
            // The IIM length cap applies to the XMP copy of a shared list too.
            let values = if shared_mapping.is_some() {
                values.iter().map(|value| truncate_to_iim(value)).collect()
            } else {
                values.clone()
            };
            xmp_values.push((*field, values));
        }
    }

    let mut changed = false;
    if !shared.is_empty() {
        changed |= set_file_info_fields(document, &shared);
    }
    if !xmp_values.is_empty() {
        changed |= set_xmp_values(document, &xmp_values);
    }
    changed
}

fn select_updates(
    current: &XmpProperties,
    template: &XmpProperties,
    mode: MergeMode,
) -> Vec<(XmpField, Vec<String>)> {
    let mut updates = Vec::new();
    for field in MANAGED {
        let values = values_of(template, field);
        let write = match mode {
            MergeMode::Append => !values.is_empty() && values_of(current, field).is_empty(),
            MergeMode::Replace => true,
            MergeMode::KeepOriginalReplaceMatching => !values.is_empty(),
        };
        if write {
            updates.push((field, values));
        }
    }
    updates
}

/// The field's values with empties dropped: the full list for
/// `Creator`/`Subject`, a single element for the scalar fields.
fn values_of(props: &XmpProperties, field: XmpField) -> Vec<String> {
    let values = match field {
        XmpField::Creator => props.creator.clone(),
        XmpField::Subject => props.subject.clone(),
        XmpField::Title => scalar(props.title.as_deref()),
        XmpField::Description => scalar(props.description.as_deref()),
        XmpField::Rights => scalar(props.rights.as_deref()),
        XmpField::Credit => scalar(props.credit.as_deref()),
        XmpField::Source => scalar(props.source.as_deref()),
        XmpField::Headline => scalar(props.headline.as_deref()),
        XmpField::Marked => props
            .marked
            .map(|marked| vec![if marked { "true" } else { "false" }.to_string()])
            .unwrap_or_default(),
    };
    values
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect()
}

fn scalar(value: Option<&str>) -> Vec<String> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| vec![value.to_string()])
        .unwrap_or_default()
}

/// The IPTC-IIM `(record, dataset)` for the six shared fields, if any.
fn iim_for(field: XmpField) -> Option<(u8, u8)> {
    Some(match field {
        XmpField::Title => (2, 5),
        XmpField::Creator => (2, 80),
        XmpField::Rights => (2, 116),
        XmpField::Description => (2, 120),
        XmpField::Credit => (2, 110),
        XmpField::Source => (2, 115),
        _ => return None,
    })
}

/// The managed fields carried as an RDF list.
fn is_list(field: XmpField) -> bool {
    matches!(field, XmpField::Creator | XmpField::Subject)
}

#[cfg(test)]
mod tests;
