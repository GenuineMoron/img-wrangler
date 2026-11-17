// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Right-click actions for Wrangler in every common file manager.
//!
//! Each file manager has a pure generator (`*_files` / `explorer_entries`) that turns a
//! `Setup` into exact file contents, so every platform's output is tested on any machine.
//! `install`, `remove` and `installed` are thin wrappers that write or delete those files.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// One pinned recipe offered in the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickAction {
    /// Recipe id passed to `quick --recipe`.
    pub id: String,
    /// Menu label.
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct Setup {
    /// The executable that handles `quick` and `open` (the Wrangler app, or the CLI).
    pub program: PathBuf,
    pub actions: Vec<QuickAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Finder,
    Explorer,
    Dolphin,
    Nautilus,
    Nemo,
    Thunar,
}

impl Target {
    pub fn label(self) -> &'static str {
        match self {
            Target::Finder => "Finder",
            Target::Explorer => "Explorer",
            Target::Dolphin => "Dolphin",
            Target::Nautilus => "Files (Nautilus)",
            Target::Nemo => "Nemo",
            Target::Thunar => "Thunar",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("{0} is not available on this system")]
    Unavailable(&'static str),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

const OPEN_LABEL: &str = "Open in Wrangler…";
const MARK: &str = "img-wrangler";

fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn on_path(cmd: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| std::env::split_paths(&paths).any(|d| d.join(cmd).is_file() || d.join(format!("{cmd}.exe")).is_file()))
}

/// The file managers this system can integrate with.
pub fn targets() -> Vec<Target> {
    if cfg!(target_os = "macos") {
        vec![Target::Finder]
    } else if cfg!(windows) {
        vec![Target::Explorer]
    } else {
        [(Target::Dolphin, "dolphin"), (Target::Nautilus, "nautilus"), (Target::Nemo, "nemo"), (Target::Thunar, "thunar")].into_iter().filter(|(_, cmd)| on_path(cmd)).map(|(t, _)| t).collect()
    }
}

/// Where the Wrangler app is usually installed, if it is.
pub fn default_program() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec!["/Applications/Wrangler.app/Contents/MacOS/wrangler".into(), home().join("Applications/Wrangler.app/Contents/MacOS/wrangler")]
    } else if cfg!(windows) {
        vec![dirs::data_local_dir().unwrap_or_default().join("Wrangler/wrangler.exe"), dirs::data_local_dir().unwrap_or_default().join("Programs/Wrangler/wrangler.exe")]
    } else {
        vec!["/usr/bin/wrangler".into(), "/usr/local/bin/wrangler".into(), home().join(".local/bin/wrangler")]
    };
    candidates.into_iter().find(|p| p.is_file())
}

/// Command words for an action, after the program.
fn args(action: Option<&QuickAction>) -> Vec<String> {
    match action {
        Some(a) => vec!["quick".into(), "--recipe".into(), a.id.clone()],
        None => vec!["open".into()],
    }
}

fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

fn desktop_quote(s: &str) -> String {
    // freedesktop Exec: double quotes, escaping \ " ` $
    let mut out = String::from("\"");
    for c in s.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn menu_items(setup: &Setup) -> Vec<(String, String, Option<&QuickAction>)> {
    // (stable id, label, action); "open" first.
    std::iter::once(("open".to_string(), OPEN_LABEL.to_string(), None)).chain(setup.actions.iter().map(|a| (a.id.clone(), a.label.clone(), Some(a)))).collect()
}

// ---------- Linux ----------

pub fn dolphin_files(setup: &Setup) -> Vec<(PathBuf, String)> {
    let items = menu_items(setup);
    let mut s = format!(
        "[Desktop Entry]\nType=Service\nMimeType=image/*;inode/directory;\nActions={};\nX-KDE-Submenu=Wrangler\nX-KDE-Priority=TopLevel\nIcon={MARK}\n",
        items.iter().map(|(id, _, _)| id.as_str()).collect::<Vec<_>>().join(";")
    );
    for (id, label, action) in &items {
        let exec = std::iter::once(desktop_quote(&setup.program.to_string_lossy())).chain(args(*action)).chain(["%F".to_string()]).collect::<Vec<_>>().join(" ");
        s.push_str(&format!("\n[Desktop Action {id}]\nName={label}\nIcon={MARK}\nExec={exec}\n"));
    }
    vec![(home().join(".local/share/kio/servicemenus/img-wrangler.desktop"), s)]
}

pub fn nautilus_files(setup: &Setup) -> Vec<(PathBuf, String)> {
    let dir = home().join(".local/share/nautilus/scripts/Wrangler");
    menu_items(setup)
        .into_iter()
        .map(|(_, label, action)| {
            let cmd = std::iter::once(sh_quote(&setup.program.to_string_lossy())).chain(args(action).iter().map(|a| sh_quote(a))).collect::<Vec<_>>().join(" ");
            (dir.join(label.replace('/', "-")), format!("#!/bin/sh\n# {MARK}: {label}\nexec {cmd} \"$@\"\n"))
        })
        .collect()
}

pub fn nemo_files(setup: &Setup) -> Vec<(PathBuf, String)> {
    let dir = home().join(".local/share/nemo/actions");
    menu_items(setup)
        .into_iter()
        .map(|(id, label, action)| {
            let exec = std::iter::once(desktop_quote(&setup.program.to_string_lossy())).chain(args(action)).chain(["%F".to_string()]).collect::<Vec<_>>().join(" ");
            let name = if action.is_some() { format!("Wrangler: {label}") } else { label.clone() };
            (dir.join(format!("{MARK}-{id}.nemo_action")), format!("[Nemo Action]\nName={name}\nComment={name}\nExec={exec}\nIcon-Name={MARK}\nSelection=notnone\nExtensions=any;\nMimetypes=image/*;inode/directory;\n"))
        })
        .collect()
}

fn thunar_path() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| home().join(".config")).join("Thunar/uca.xml")
}

/// Merge Wrangler's actions into Thunar's `uca.xml`, replacing any older Wrangler actions and
/// leaving the user's own untouched.
pub fn thunar_merge(existing: Option<&str>, setup: Option<&Setup>) -> String {
    let base = existing.filter(|s| s.contains("<actions")).unwrap_or("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n</actions>\n");
    // Drop every <action>…</action> block that carries our unique-id, keep everything else.
    let mut doc = String::with_capacity(base.len());
    let mut rest = base;
    while let Some(start) = rest.find("<action>") {
        let Some(len) = rest[start..].find("</action>") else { break };
        let end = start + len + "</action>".len();
        let block = &rest[start..end];
        if block.contains(&format!("<unique-id>{MARK}-")) {
            doc.push_str(rest[..start].trim_end_matches([' ', '\t']));
            rest = rest[end..].strip_prefix('\n').unwrap_or(&rest[end..]);
        } else {
            doc.push_str(&rest[..end]);
            rest = &rest[end..];
        }
    }
    doc.push_str(rest);
    if let Some(setup) = setup {
        let mut add = String::new();
        for (id, label, action) in menu_items(setup) {
            let cmd = std::iter::once(desktop_quote(&setup.program.to_string_lossy())).chain(args(action)).chain(["%F".to_string()]).collect::<Vec<_>>().join(" ");
            let name = if action.is_some() { format!("Wrangler: {label}") } else { label };
            add.push_str(&format!(
                "<action>\n\t<icon>{MARK}</icon>\n\t<name>{}</name>\n\t<submenu></submenu>\n\t<unique-id>{MARK}-{}</unique-id>\n\t<command>{}</command>\n\t<description></description>\n\t<range>*</range>\n\t<patterns>*</patterns>\n\t<directories/>\n\t<image-files/>\n</action>\n",
                xml_escape(&name), xml_escape(&id), xml_escape(&cmd)
            ));
        }
        let at = doc.rfind("</actions>").unwrap_or(doc.len());
        doc.insert_str(at, &add);
    }
    doc
}

// ---------- macOS ----------

fn finder_dir() -> PathBuf {
    home().join("Library/Services")
}

fn plist_escape(s: &str) -> String {
    xml_escape(s)
}

/// An Automator Quick Action (a Services workflow) that runs one command on the selected
/// images or folders.
pub fn finder_files(setup: &Setup) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for (_, label, action) in menu_items(setup) {
        let title = if action.is_some() { format!("Wrangler: {label}") } else { label.clone() };
        let bundle = finder_dir().join(format!("{}.workflow", title.replace(": ", " - ").replace(['/', ':'], "-")));
        let cmd = std::iter::once(sh_quote(&setup.program.to_string_lossy())).chain(args(action).iter().map(|a| sh_quote(a))).collect::<Vec<_>>().join(" ");
        let script = format!("# {MARK}\n{cmd} \"$@\" > /dev/null 2>&1 &");
        let info = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSBackgroundColorName</key>
			<string>background</string>
			<key>NSIconName</key>
			<string>NSActionTemplate</string>
			<key>NSMenuItem</key>
			<dict>
				<key>default</key>
				<string>{title}</string>
			</dict>
			<key>NSMessage</key>
			<string>runWorkflowAsService</string>
			<key>NSRequiredContext</key>
			<dict>
				<key>NSApplicationIdentifier</key>
				<string>com.apple.finder</string>
			</dict>
			<key>NSSendFileTypes</key>
			<array>
				<string>public.image</string>
				<string>public.folder</string>
			</array>
		</dict>
	</array>
</dict>
</plist>
"#,
            title = plist_escape(&title)
        );
        let wflow = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>528</string>
	<key>AMApplicationVersion</key>
	<string>2.10</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>AMParameterProperties</key>
				<dict>
					<key>COMMAND_STRING</key>
					<dict/>
					<key>CheckedForUserDefaultShell</key>
					<dict/>
					<key>inputMethod</key>
					<dict/>
					<key>shell</key>
					<dict/>
					<key>source</key>
					<dict/>
				</dict>
				<key>AMProvides</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>{script}</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>1</integer>
					<key>shell</key>
					<string>/bin/sh</string>
					<key>source</key>
					<string></string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>8A9C1C6E-2F0B-4E7A-9C33-0D6E0A6B1F01</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
				</array>
				<key>OutputUUID</key>
				<string>5F0C2B8D-7E1A-4B33-8E21-6C1D9A2E3F02</string>
				<key>UUID</key>
				<string>3B7E9D10-4C2A-4F55-9A11-2E8F6B0C4D03</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
				<key>arguments</key>
				<dict/>
				<key>isViewVisible</key>
				<integer>1</integer>
			</dict>
			<key>isViewVisible</key>
			<integer>1</integer>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>applicationBundleID</key>
		<string>com.apple.finder</string>
		<key>serviceApplicationBundleID</key>
		<string>com.apple.finder</string>
		<key>serviceInputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<integer>0</integer>
		<key>workflowTypeIdentifier</key>
		<string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>
"#,
            script = plist_escape(&script)
        );
        out.push((bundle.join("Contents/Info.plist"), info));
        out.push((bundle.join("Contents/document.wflow"), wflow));
    }
    out
}

// ---------- Windows ----------

/// One registry value: key under `HKEY_CURRENT_USER`, value name (`""` for the default), data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegEntry {
    pub key: String,
    pub name: String,
    pub value: String,
}

const EXPLORER_ROOTS: [&str; 2] = [r"Software\Classes\SystemFileAssociations\image\shell\Wrangler", r"Software\Classes\Directory\shell\Wrangler"];

/// A cascading "Wrangler" menu for images and folders. `MultiSelectModel=Player` keeps the
/// menu visible when many files are selected; the app folds the per-file launches into one batch.
pub fn explorer_entries(setup: &Setup) -> Vec<RegEntry> {
    let exe = setup.program.to_string_lossy().to_string();
    let mut out = Vec::new();
    for root in EXPLORER_ROOTS {
        let e = |key: &str, name: &str, value: String| RegEntry { key: key.to_string(), name: name.to_string(), value };
        out.push(e(root, "MUIVerb", "Wrangler".into()));
        out.push(e(root, "SubCommands", String::new()));
        out.push(e(root, "Icon", exe.clone()));
        out.push(e(root, "MultiSelectModel", "Player".into()));
        for (n, (id, label, action)) in menu_items(setup).into_iter().enumerate() {
            let key = format!(r"{root}\shell\{n:02}-{id}");
            let cmd = std::iter::once(format!("\"{exe}\"")).chain(args(action)).chain(["\"%1\"".to_string()]).collect::<Vec<_>>().join(" ");
            out.push(e(&key, "MUIVerb", label));
            out.push(e(&key, "MultiSelectModel", "Player".into()));
            out.push(e(&format!(r"{key}\command"), "", cmd));
        }
    }
    out
}

/// The same entries as a `.reg` file, for people who prefer to inspect or import by hand.
pub fn explorer_reg_text(setup: &Setup) -> String {
    let mut s = String::from("Windows Registry Editor Version 5.00\n");
    let mut last = String::new();
    for e in explorer_entries(setup) {
        if e.key != last {
            s.push_str(&format!("\n[HKEY_CURRENT_USER\\{}]\n", e.key));
            last = e.key.clone();
        }
        let name = if e.name.is_empty() { "@".to_string() } else { format!("\"{}\"", e.name) };
        s.push_str(&format!("{name}=\"{}\"\n", e.value.replace('\\', "\\\\").replace('"', "\\\"")));
    }
    s
}

// ---------- install / remove ----------

fn write_all(files: &[(PathBuf, String)], executable: bool) -> Result<Vec<PathBuf>> {
    for (path, text) in files {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::Io(dir.into(), e))?;
        }
        fs::write(path, text).map_err(|e| Error::Io(path.clone(), e))?;
        #[cfg(unix)]
        if executable {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|e| Error::Io(path.clone(), e))?;
        }
        #[cfg(not(unix))]
        let _ = executable;
    }
    Ok(files.iter().map(|(p, _)| p.clone()).collect())
}

fn remove_matching(dir: &Path, is_ours: impl Fn(&Path) -> bool) -> Result<()> {
    let Ok(entries) = fs::read_dir(dir) else { return Ok(()) };
    for e in entries.flatten() {
        let p = e.path();
        if is_ours(&p) {
            let r = if p.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) };
            r.map_err(|err| Error::Io(p.clone(), err))?;
        }
    }
    Ok(())
}

fn file_mentions_mark(p: &Path) -> bool {
    fs::read_to_string(p).is_ok_and(|t| t.contains(MARK))
}

pub fn install(target: Target, setup: &Setup) -> Result<Vec<PathBuf>> {
    remove(target)?;
    match target {
        Target::Dolphin => write_all(&dolphin_files(setup), true),
        Target::Nautilus => write_all(&nautilus_files(setup), true),
        Target::Nemo => write_all(&nemo_files(setup), false),
        Target::Thunar => {
            let path = thunar_path();
            let merged = thunar_merge(fs::read_to_string(&path).ok().as_deref(), Some(setup));
            write_all(&[(path, merged)], false)
        }
        Target::Finder => {
            if !cfg!(target_os = "macos") {
                return Err(Error::Unavailable("Finder"));
            }
            let files = write_all(&finder_files(setup), false)?;
            let _ = std::process::Command::new("/System/Library/CoreServices/pbs").arg("-flush").status();
            Ok(files)
        }
        Target::Explorer => install_explorer(setup),
    }
}

pub fn remove(target: Target) -> Result<()> {
    match target {
        Target::Dolphin => remove_matching(&home().join(".local/share/kio/servicemenus"), |p| p.file_name().is_some_and(|n| n == "img-wrangler.desktop")),
        Target::Nautilus => remove_matching(&home().join(".local/share/nautilus/scripts"), |p| p.file_name().is_some_and(|n| n == "Wrangler")),
        Target::Nemo => remove_matching(&home().join(".local/share/nemo/actions"), |p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(MARK))),
        Target::Thunar => {
            let path = thunar_path();
            match fs::read_to_string(&path) {
                Ok(text) if text.contains(MARK) => fs::write(&path, thunar_merge(Some(&text), None)).map_err(|e| Error::Io(path, e)),
                _ => Ok(()),
            }
        }
        Target::Finder => remove_matching(&finder_dir(), |p| p.extension().is_some_and(|x| x == "workflow") && file_mentions_mark(&p.join("Contents/document.wflow"))),
        Target::Explorer => remove_explorer(),
    }
}

pub fn installed(target: Target) -> bool {
    match target {
        Target::Dolphin => home().join(".local/share/kio/servicemenus/img-wrangler.desktop").exists(),
        Target::Nautilus => home().join(".local/share/nautilus/scripts/Wrangler").exists(),
        Target::Nemo => fs::read_dir(home().join(".local/share/nemo/actions")).is_ok_and(|d| d.flatten().any(|e| e.file_name().to_string_lossy().starts_with(MARK))),
        Target::Thunar => file_mentions_mark(&thunar_path()),
        Target::Finder => fs::read_dir(finder_dir()).is_ok_and(|d| d.flatten().any(|e| file_mentions_mark(&e.path().join("Contents/document.wflow")))),
        Target::Explorer => explorer_installed(),
    }
}

#[cfg(windows)]
fn install_explorer(setup: &Setup) -> Result<Vec<PathBuf>> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for e in explorer_entries(setup) {
        let (key, _) = hkcu.create_subkey(&e.key).map_err(|err| Error::Other(format!("{}: {err}", e.key)))?;
        key.set_value(&e.name, &e.value).map_err(|err| Error::Other(format!("{}: {err}", e.key)))?;
    }
    Ok(EXPLORER_ROOTS.iter().map(PathBuf::from).collect())
}

#[cfg(windows)]
fn remove_explorer() -> Result<()> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for root in EXPLORER_ROOTS {
        let _ = hkcu.delete_subkey_all(root);
    }
    Ok(())
}

#[cfg(windows)]
fn explorer_installed() -> bool {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    RegKey::predef(HKEY_CURRENT_USER).open_subkey(EXPLORER_ROOTS[0]).is_ok()
}

#[cfg(not(windows))]
fn install_explorer(_: &Setup) -> Result<Vec<PathBuf>> {
    Err(Error::Unavailable("Explorer"))
}

#[cfg(not(windows))]
fn remove_explorer() -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
fn explorer_installed() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Setup {
        Setup { program: PathBuf::from("/opt/Wrangler/wrangler"), actions: vec![QuickAction { id: "shopify-product".into(), label: "Shopify product".into() }] }
    }

    #[test]
    fn dolphin_service_menu() {
        let (path, text) = &dolphin_files(&setup())[0];
        assert!(path.ends_with(".local/share/kio/servicemenus/img-wrangler.desktop"));
        assert!(text.contains("Actions=open;shopify-product;"));
        assert!(text.contains("X-KDE-Submenu=Wrangler"));
        assert!(text.contains("Exec=\"/opt/Wrangler/wrangler\" quick --recipe shopify-product %F"));
        assert!(text.contains("Exec=\"/opt/Wrangler/wrangler\" open %F"));
    }

    #[test]
    fn nautilus_scripts_pass_selection() {
        let files = nautilus_files(&setup());
        assert_eq!(files.len(), 2);
        assert!(files[1].0.ends_with("Wrangler/Shopify product"));
        assert!(files[1].1.contains("exec '/opt/Wrangler/wrangler' 'quick' '--recipe' 'shopify-product' \"$@\""));
    }

    #[test]
    fn nemo_actions() {
        let files = nemo_files(&setup());
        assert!(files[1].1.contains("Name=Wrangler: Shopify product"));
        assert!(files[1].1.contains("Mimetypes=image/*;inode/directory;"));
    }

    #[test]
    fn thunar_merge_keeps_user_actions() {
        let existing = "<?xml version=\"1.0\"?>\n<actions>\n<action>\n\t<name>Open Terminal</name>\n\t<unique-id>123</unique-id>\n</action>\n</actions>\n";
        let merged = thunar_merge(Some(existing), Some(&setup()));
        assert!(merged.contains("Open Terminal"));
        assert_eq!(merged.matches("<unique-id>img-wrangler-").count(), 2);
        let again = thunar_merge(Some(&merged), Some(&setup()));
        assert_eq!(again.matches("<unique-id>img-wrangler-").count(), 2, "re-installing doesn't duplicate");
        let removed = thunar_merge(Some(&again), None);
        assert!(!removed.contains("img-wrangler"));
        assert!(removed.contains("Open Terminal"));
    }

    #[test]
    fn finder_quick_actions() {
        let files = finder_files(&setup());
        assert_eq!(files.len(), 4, "Info.plist and document.wflow for each of two actions");
        let find = |suffix: &str| files.iter().find(|(p, _)| p.to_string_lossy().ends_with(suffix)).map(|f| f.1.clone()).unwrap_or_else(|| panic!("missing {suffix}"));
        let info = find("Wrangler - Shopify product.workflow/Contents/Info.plist");
        assert!(info.contains("<string>Wrangler: Shopify product</string>"));
        assert!(info.contains("<string>public.image</string>"));
        let wflow = find("Wrangler - Shopify product.workflow/Contents/document.wflow");
        assert!(wflow.contains("'/opt/Wrangler/wrangler' 'quick' '--recipe' 'shopify-product' &quot;$@&quot;"));
        assert!(wflow.contains("com.apple.Automator.servicesMenu"));
        assert!(find("Open in Wrangler….workflow/Contents/document.wflow").contains("'open'"));
    }

    #[test]
    fn explorer_cascading_menu() {
        let reg = explorer_reg_text(&setup());
        assert!(reg.contains(r"[HKEY_CURRENT_USER\Software\Classes\SystemFileAssociations\image\shell\Wrangler\shell\01-shopify-product\command]"));
        assert!(reg.contains(r#"@="\"/opt/Wrangler/wrangler\" quick --recipe shopify-product \"%1\"""#));
        assert!(reg.contains("\"MultiSelectModel\"=\"Player\""));
        assert!(reg.contains(r"Directory\shell\Wrangler"));
    }
}
