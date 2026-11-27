// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

// Wrangler in Finder's own right-click menu: a "Wrangler" entry with the pinned recipes and
// "Open in Wrangler…", shown when the selection holds an image or a folder.
//
// Finder runs this as a Finder Sync extension, in a sandbox. It does no image work. It reads the
// menu the app wrote (finder/menu.json in Wrangler's Application Support folder), and when an
// item is chosen it writes a request beside it (finder/requests/<id>.json) and wakes the app
// with img-wrangler://finder. The address carries nothing, so nothing else can ask Wrangler to
// run by opening it: the app only acts on request files, which only this user's programs can
// write. Built by scripts/finder-extension.mjs, with no Xcode project.

import Cocoa
import FinderSync

/// What the app wrote for us to show (finder/menu.json).
struct WranglerMenu: Decodable {
    struct Recipe: Decodable {
        let id: String
        let label: String
    }
    /// Image file extensions, lower case.
    let extensions: [String]
    /// Label of the item that loads the selection into the main window.
    let open: String
    let recipes: [Recipe]

    static func load(from folder: URL) -> WranglerMenu? {
        guard let data = try? Data(contentsOf: folder.appendingPathComponent("menu.json")) else { return nil }
        return try? JSONDecoder().decode(WranglerMenu.self, from: data)
    }

    /// Whether to show for this selection: an image by its extension, or a folder. The sandbox
    /// can't look at the files themselves, so a name with no extension counts as a folder.
    /// Wrangler checks properly when it runs.
    func applies(to urls: [URL]) -> Bool {
        urls.contains { url in
            let ext = url.pathExtension.lowercased()
            return url.hasDirectoryPath || ext.isEmpty || extensions.contains(ext)
        }
    }

    /// "Wrangler ▸" with the recipes, then the open item. An item's tag is its place in the
    /// list, as tags are all that survive the trip back from Finder. The last one opens. No
    /// dividing line: Finder draws one of ours as an empty row.
    func build(action: Selector, hat: NSImage?) -> NSMenu {
        let sub = NSMenu(title: "Wrangler")
        for (i, recipe) in recipes.enumerated() {
            let item = NSMenuItem(title: recipe.label, action: action, keyEquivalent: "")
            item.tag = i
            sub.addItem(item)
        }
        let last = NSMenuItem(title: open, action: action, keyEquivalent: "")
        last.tag = recipes.count
        sub.addItem(last)

        let top = NSMenuItem(title: "Wrangler", action: nil, keyEquivalent: "")
        top.image = hat.map(WranglerMenu.icon)
        top.submenu = sub
        let root = NSMenu(title: "")
        root.addItem(top)
        return root
    }

    /// The hat as a menu icon. Finder draws every icon in a square, which squashed the wide
    /// hat, so it's drawn at its own shape in the middle of a square of plain pixels.
    static func icon(_ hat: NSImage) -> NSImage {
        let side = 16, scale = 2
        let icon = NSImage(size: NSSize(width: side, height: side))
        guard hat.size.width > 0, hat.size.height > 0,
              let pixels = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: side * scale, pixelsHigh: side * scale, bitsPerSample: 8, samplesPerPixel: 4,
                                            hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)
        else { return hat }
        // Sized in points before drawing starts, so the drawing below is in points too.
        pixels.size = icon.size
        guard let context = NSGraphicsContext(bitmapImageRep: pixels) else { return hat }
        let fit = min(CGFloat(side) / hat.size.width, CGFloat(side) / hat.size.height)
        let (w, h) = (hat.size.width * fit, hat.size.height * fit)
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = context
        hat.draw(in: NSRect(x: (CGFloat(side) - w) / 2, y: (CGFloat(side) - h) / 2, width: w, height: h))
        NSGraphicsContext.restoreGraphicsState()
        icon.addRepresentation(pixels)
        icon.isTemplate = true
        return icon
    }

    /// What choosing the item with this tag asks for: a recipe to run on the paths, or, with no
    /// recipe, to open them in the main window.
    func request(tag: Int, urls: [URL]) -> [String: Any] {
        var request: [String: Any] = ["paths": urls.map(\.path)]
        if tag >= 0, tag < recipes.count { request["recipe"] = recipes[tag].id }
        return request
    }

    /// Hand a request to the app: written whole, then renamed into place, so the app never reads
    /// half of one.
    @discardableResult
    static func write(_ request: [String: Any], in folder: URL) throws -> URL {
        let requests = folder.appendingPathComponent("requests")
        let name = UUID().uuidString
        let partial = requests.appendingPathComponent(name + ".tmp")
        let whole = requests.appendingPathComponent(name + ".json")
        try FileManager.default.createDirectory(at: requests, withIntermediateDirectories: true)
        try JSONSerialization.data(withJSONObject: request).write(to: partial)
        try FileManager.default.moveItem(at: partial, to: whole)
        return whole
    }
}

final class FinderSync: FIFinderSync {
    /// ~/Library/Application Support/img-wrangler/finder. HOME is the sandbox's own container
    /// here, so the real home comes from the user database.
    private let folder: URL = {
        let home = getpwuid(getuid()).flatMap { $0.pointee.pw_dir }.map { String(cString: $0) } ?? NSHomeDirectory()
        return URL(fileURLWithPath: home).appendingPathComponent("Library/Application Support/img-wrangler/finder")
    }()
    /// The menu last shown, so a chosen item's tag finds its recipe.
    private var shown: WranglerMenu?

    override init() {
        super.init()
        // Offer the menu everywhere. The top of the disk covers every drive mounted under it,
        // and naming the drives as well made Finder show the menu twice for files on them.
        FIFinderSyncController.default().directoryURLs = [URL(fileURLWithPath: "/")]
    }

    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        guard menuKind == .contextualMenuForItems,
              let menu = WranglerMenu.load(from: folder),
              let urls = FIFinderSyncController.default().selectedItemURLs(),
              menu.applies(to: urls)
        else { return nil }
        shown = menu
        return menu.build(action: #selector(choose(_:)), hat: Bundle.main.image(forResource: "hat"))
    }

    @objc private func choose(_ sender: NSMenuItem) {
        guard let menu = shown,
              let urls = FIFinderSyncController.default().selectedItemURLs(), !urls.isEmpty
        else { return }
        do {
            try WranglerMenu.write(menu.request(tag: sender.tag, urls: urls), in: folder)
        } catch {
            NSLog("Wrangler: couldn't hand the selection over: \(error.localizedDescription)")
            return
        }
        if let wake = URL(string: "img-wrangler://finder") {
            NSWorkspace.shared.open(wake)
        }
    }
}
