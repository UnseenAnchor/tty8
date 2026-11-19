//! Which glyph a file tree row wears, and where its bytes live.
//!
//! The glyphs are the [Material Icon Theme]'s, vendored under
//! `assets/icons/material` with their MIT licence beside them. `build.rs` turns
//! the theme's association table into the `FILES`/`FILE_NAMES`/... statics this
//! module searches, so a bump is a matter of dropping in a new `icons/` folder
//! and `material-icons.json`.
//!
//! These are full-colour glyphs, which is the whole reason they are drawn with
//! `img()` rather than `Icon`: `svg()` paints an alpha mask tinted with the text
//! colour, so every icon would come out flat and monochrome.
//!
//! [Material Icon Theme]: https://github.com/material-extensions/vscode-material-icon-theme

use gpui::{AnyElement, ImageSource, IntoElement as _, Resource, Styled as _, img, px};

include!(concat!(env!("OUT_DIR"), "/material_icons.rs"));

/// The glyph for `name`. Directories answer by name and by whether they are
/// open; files answer by name first and then by their longest extension.
pub(crate) fn icon_for(name: &str, is_dir: bool, expanded: bool) -> &'static str {
    if is_dir {
        folder_icon(name, expanded)
    } else {
        file_icon(name)
    }
}

/// The glyph as an element of the box a row gives it.
pub(crate) fn glyph(path: &'static str, size: f32) -> AnyElement {
    img(ImageSource::Resource(Resource::Embedded(path.into())))
        .flex_none()
        .w(px(size))
        .h(px(size))
        .into_any_element()
}

fn folder_icon(name: &str, expanded: bool) -> &'static str {
    // The theme's closed and open tables hold the same folder names, so one
    // lookup against whichever state the row is in is enough.
    let (table, base) = if expanded {
        (FOLDER_NAMES_OPEN, DEFAULT_FOLDER_OPEN)
    } else {
        (FOLDER_NAMES, DEFAULT_FOLDER)
    };
    lookup(table, &name.to_lowercase()).unwrap_or(base)
}

fn file_icon(name: &str) -> &'static str {
    if let Some(path) = lookup(FILE_NAMES_EXACT, name) {
        return path;
    }
    let folded = name.to_lowercase();
    if let Some(path) = lookup(FILE_NAMES, &folded) {
        return path;
    }
    // `app.d.ts` has to reach `d.ts` before it reaches `ts`, so every dot in turn
    // starts a candidate and the first hit wins — which is the order the theme's
    // own matching uses.
    let mut from = 0;
    while let Some(dot) = folded[from..].find('.') {
        from += dot + 1;
        let Some(extension) = folded.get(from..).filter(|rest| !rest.is_empty()) else {
            break;
        };
        if let Some(path) = lookup(FILE_EXTENSIONS, extension) {
            return path;
        }
    }
    DEFAULT_FILE
}

fn lookup(table: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table
        .binary_search_by(|(candidate, _)| (*candidate).cmp(key))
        .ok()
        .map(|index| table[index].1)
}

fn lookup_bytes(path: &str) -> Option<&'static [u8]> {
    FILES
        .binary_search_by(|(candidate, _)| (*candidate).cmp(path))
        .ok()
        .map(|index| FILES[index].1)
}

/// A vendored glyph's bytes, for [`crate::ui::assets::Assets::load`] — borrowed,
/// since they live in the binary.
pub(crate) fn asset(path: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
    lookup_bytes(path).map(std::borrow::Cow::Borrowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name_of(path: &str) -> &str {
        path.rsplit('/').next().unwrap_or(path)
    }

    #[test]
    fn a_source_file_wears_the_glyph_of_its_language() {
        assert_eq!(name_of(icon_for("main.rs", false, false)), "rust.svg");
        assert_eq!(name_of(icon_for("view.tsx", false, false)), "react_ts.svg");
        assert_eq!(name_of(icon_for("notes.md", false, false)), "markdown.svg");
    }

    #[test]
    fn a_two_part_extension_is_tried_before_its_tail() {
        // `ts` alone is the TypeScript glyph; `d.ts` has one of its own.
        assert_ne!(
            icon_for("index.d.ts", false, false),
            icon_for("index.ts", false, false)
        );
        assert_eq!(
            name_of(icon_for("index.d.ts", false, false)),
            "typescript-def.svg"
        );
    }

    #[test]
    fn a_known_file_name_beats_its_extension() {
        assert_eq!(
            name_of(icon_for("docker-compose.yml", false, false)),
            "docker.svg"
        );
        assert_eq!(name_of(icon_for("yarn.lock", false, false)), "yarn.svg");
    }

    #[test]
    fn the_few_uppercase_names_match_as_written() {
        // `APKBUILD` is spelled that way in the theme, and so is its glyph; the
        // folded table is what catches `apkbuild`, `ApkBuild` and friends.
        assert_eq!(name_of(icon_for("APKBUILD", false, false)), "console.svg");
    }

    #[test]
    fn an_unknown_extension_falls_back_to_the_plain_file() {
        assert_eq!(icon_for("mystery.zzz", false, false), DEFAULT_FILE);
        assert_eq!(icon_for("no-extension-at-all", false, false), DEFAULT_FILE);
        // A trailing dot must not slice past the end.
        assert_eq!(icon_for("trailing.", false, false), DEFAULT_FILE);
    }

    #[test]
    fn a_folder_answers_by_its_name_and_its_state() {
        let closed = icon_for("src", true, false);
        let open = icon_for("src", true, true);
        assert_eq!(name_of(closed), "folder-src.svg");
        assert_eq!(name_of(open), "folder-src-open.svg");
        assert_ne!(closed, open);
    }

    #[test]
    fn an_unknown_folder_falls_back_to_the_plain_folder() {
        assert_eq!(icon_for("no-such-folder-name", true, false), DEFAULT_FOLDER);
        assert_eq!(
            icon_for("no-such-folder-name", true, true),
            DEFAULT_FOLDER_OPEN
        );
    }

    #[test]
    fn a_root_folder_is_looked_up_like_any_other_folder() {
        // The theme's `rootFolder` glyph is a bare marker rather than a folder,
        // so the tree asks about the root by name like it does for any folder.
        assert_eq!(icon_for("tty8", true, false), DEFAULT_FOLDER);
        assert_eq!(icon_for("tty8", true, true), DEFAULT_FOLDER_OPEN);
        assert_eq!(
            icon_for("src", true, false),
            lookup(FOLDER_NAMES, "src").expect("src wears a folder of its own")
        );
    }

    #[test]
    fn every_table_entry_points_at_a_bundled_glyph() {
        // A path that is named but not embedded would render as a hole in the
        // tree with nothing in the logs to say why.
        let tables: [&[(&str, &str)]; 5] = [
            FILE_NAMES_EXACT,
            FILE_NAMES,
            FILE_EXTENSIONS,
            FOLDER_NAMES,
            FOLDER_NAMES_OPEN,
        ];
        for table in tables {
            for (key, path) in table {
                assert!(lookup_bytes(path).is_some(), "{key} names {path}");
            }
        }
        for path in [DEFAULT_FILE, DEFAULT_FOLDER, DEFAULT_FOLDER_OPEN] {
            assert!(lookup_bytes(path).is_some(), "{path} is not embedded");
        }
    }

    #[test]
    fn the_asset_source_serves_a_vendored_glyph() {
        // The whole chain in one call: `img()` asks the asset source for the
        // path a row chose, so a path that only exists in the table would draw
        // an empty box.
        use gpui::AssetSource as _;

        let bytes = crate::ui::assets::Assets
            .load(icon_for("main.rs", false, false))
            .expect("the asset source answers")
            .expect("the glyph is embedded");
        assert!(bytes.windows(4).any(|window| window == b"<svg"));
    }

    #[test]
    fn the_glyph_table_is_sorted_for_the_binary_search() {
        assert!(FILES.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for table in [
            FILE_NAMES_EXACT,
            FILE_NAMES,
            FILE_EXTENSIONS,
            FOLDER_NAMES,
            FOLDER_NAMES_OPEN,
        ] {
            assert!(table.windows(2).all(|pair| pair[0].0 < pair[1].0));
        }
    }
}
