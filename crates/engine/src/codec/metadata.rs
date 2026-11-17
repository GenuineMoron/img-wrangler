// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! What metadata survives into the output: none, the safe part (no location), or all of it.
//! Orientation is always reset, because pixels are already upright.

use exif::{experimental::Writer, Field, In, Tag, Value};
use serde::{Deserialize, Serialize};

use super::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum KeepMeta {
    /// Remove everything. Smallest and most private.
    #[default]
    None,
    /// Keep camera, date and copyright; drop GPS location and the embedded thumbnail.
    Safe,
    /// Keep all EXIF, location included.
    All,
}

/// Rewrite the source EXIF for output, or `None` when nothing should be kept.
pub fn exif_for_output(source: Option<&[u8]>, keep: KeepMeta) -> Option<Vec<u8>> {
    if keep == KeepMeta::None {
        return None;
    }
    let raw = source?;
    let raw = raw.strip_prefix(b"Exif\0\0").unwrap_or(raw);
    let parsed = exif::Reader::new().read_raw(raw.to_vec()).ok()?;
    let upright = Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Value::Short(vec![1]) };
    let kept: Vec<&Field> = parsed
        .fields()
        .filter(|f| f.ifd_num == In::PRIMARY)
        .filter(|f| f.tag != Tag::Orientation)
        .filter(|f| keep == KeepMeta::All || f.tag.context() != exif::Context::Gps)
        .collect();
    let mut w = Writer::new();
    w.push_field(&upright);
    for f in &kept {
        w.push_field(f);
    }
    let mut out = std::io::Cursor::new(Vec::new());
    w.write(&mut out, parsed.little_endian()).ok()?;
    Some(out.into_inner())
}

/// Attach EXIF to an encoded file where the container supports it (JPEG, PNG, WebP).
pub fn attach_exif(bytes: Vec<u8>, format: Format, exif: Option<&[u8]>) -> Vec<u8> {
    use img_parts::{Bytes, ImageEXIF};
    let Some(exif) = exif else { return bytes };
    let exif = Bytes::copy_from_slice(exif);
    let input = Bytes::from(bytes.clone());
    let out = match format {
        Format::Jpeg => img_parts::jpeg::Jpeg::from_bytes(input).ok().map(|mut j| { j.set_exif(Some(exif)); j.encoder().bytes() }),
        Format::Png => img_parts::png::Png::from_bytes(input).ok().map(|mut p| { p.set_exif(Some(exif)); p.encoder().bytes() }),
        Format::Webp => img_parts::webp::WebP::from_bytes(input).ok().map(|mut p| { p.set_exif(Some(exif)); p.encoder().bytes() }),
        _ => None,
    };
    out.map(|b| b.to_vec()).unwrap_or(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_exif() -> Vec<u8> {
        let fields = [
            Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Value::Short(vec![6]) },
            Field { tag: Tag::Make, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"Wrangler".to_vec()]) },
            Field { tag: Tag::GPSLatitudeRef, ifd_num: In::PRIMARY, value: Value::Ascii(vec![b"S".to_vec()]) },
        ];
        let mut w = Writer::new();
        fields.iter().for_each(|f| w.push_field(f));
        let mut out = std::io::Cursor::new(Vec::new());
        w.write(&mut out, false).unwrap();
        out.into_inner()
    }

    #[test]
    fn none_keeps_nothing() {
        assert!(exif_for_output(Some(&sample_exif()), KeepMeta::None).is_none());
    }

    #[test]
    fn safe_drops_location_and_resets_orientation() {
        let out = exif_for_output(Some(&sample_exif()), KeepMeta::Safe).unwrap();
        let back = exif::Reader::new().read_raw(out).unwrap();
        assert!(back.get_field(Tag::GPSLatitudeRef, In::PRIMARY).is_none());
        assert!(back.get_field(Tag::Make, In::PRIMARY).is_some());
        assert_eq!(back.get_field(Tag::Orientation, In::PRIMARY).unwrap().value.get_uint(0), Some(1));
    }
}
