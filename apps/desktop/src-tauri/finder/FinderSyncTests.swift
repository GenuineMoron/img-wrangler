// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

// Checks the Finder extension's menu and requests without Finder:
//   node scripts/finder-extension.mjs --test

import Cocoa

@main
enum Tests {
    static var failed = 0

    static func check(_ ok: Bool, _ what: String) {
        if !ok {
            failed += 1
            print("FAILED: \(what)")
        }
    }

    static func main() throws {
        let folder = FileManager.default.temporaryDirectory.appendingPathComponent("wrangler-finder-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: folder) }

        check(WranglerMenu.load(from: folder) == nil, "no menu file, no menu")
        let json = #"{"extensions":["jpg","png"],"open":"Open in Wrangler…","recipes":[{"id":"web-ready","label":"Web ready"},{"id":"just-compress","label":"Just compress"}]}"#
        try json.write(to: folder.appendingPathComponent("menu.json"), atomically: true, encoding: .utf8)
        guard let menu = WranglerMenu.load(from: folder) else {
            print("FAILED: the menu file didn't load")
            exit(1)
        }

        let file = { (p: String) in URL(fileURLWithPath: p, isDirectory: false) }
        let dir = URL(fileURLWithPath: "/Users/x/Pictures/shoot", isDirectory: true)
        check(menu.applies(to: [file("/a/photo.JPG")]), "shows for an image, whatever the case of its extension")
        check(menu.applies(to: [dir]), "shows for a folder")
        check(menu.applies(to: [file("/a/notes.txt"), file("/a/b.png")]), "shows when any of the selection is an image")
        check(!menu.applies(to: [file("/a/notes.txt"), file("/a/movie.mov")]), "stays out of the way for other files")
        check(!menu.applies(to: []), "nothing selected, nothing shown")

        let built = menu.build(action: #selector(NSObject.description), hat: nil)
        check(built.items.map(\.title) == ["Wrangler"], "one entry in Finder's menu")
        let sub = built.items[0].submenu?.items ?? []
        check(sub.map(\.title) == ["Web ready", "Just compress", "", "Open in Wrangler…"], "recipes, a line, then open: \(sub.map(\.title))")
        check(sub.filter { !$0.isSeparatorItem }.map(\.tag) == [0, 1, 2], "tags count up")

        let urls = [file("/a/one.jpg"), file("/a/two with space.png")]
        let run = menu.request(tag: 1, urls: urls)
        check(run["recipe"] as? String == "just-compress", "a recipe's tag asks for that recipe")
        check(run["paths"] as? [String] == ["/a/one.jpg", "/a/two with space.png"], "with the selected paths")
        check(menu.request(tag: 2, urls: urls)["recipe"] == nil, "the last tag opens, with no recipe")

        let written = try WranglerMenu.write(run, in: folder)
        let back = try JSONSerialization.jsonObject(with: Data(contentsOf: written)) as? [String: Any]
        check(back?["recipe"] as? String == "just-compress" && (back?["paths"] as? [String])?.count == 2, "the request reads back")
        let left = try FileManager.default.contentsOfDirectory(atPath: folder.appendingPathComponent("requests").path)
        check(left.count == 1 && left[0].hasSuffix(".json"), "one whole request, nothing half written: \(left)")

        print(failed == 0 ? "Finder extension: 14 checks passed" : "Finder extension: \(failed) failed")
        exit(failed == 0 ? 0 : 1)
    }
}
