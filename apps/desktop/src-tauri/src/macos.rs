// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Images dropped on Wrangler's menu bar icon run straight away, with the recipe chosen under
//! "Images dropped on the hat" in its menu.
//!
//! The tray library draws the icon in a status bar button and takes clicks in a view on top of
//! it. This adds a view underneath that takes drops only: AppKit gives a drag to the view under
//! the pointer that registered for it, so clicks still reach the view above unchanged.

use std::path::PathBuf;

use objc2::rc::Retained;
use objc2::runtime::{Bool, NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, ClassType, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSAutoresizingMaskOptions, NSDragOperation, NSDraggingDestination, NSDraggingInfo, NSPasteboardTypeFileURL, NSStatusItem, NSView, NSWindowOrderingMode};
use objc2_foundation::{NSArray, NSURL};
use tauri::AppHandle;

pub struct Ivars {
    app: AppHandle,
}

define_class!(
    // SAFETY: NSView has no subclassing requirements, and DropView doesn't implement Drop.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "WranglerMenuBarDrop"]
    #[ivars = Ivars]
    struct DropView;

    unsafe impl NSObjectProtocol for DropView {}

    unsafe impl NSDraggingDestination for DropView {
        #[unsafe(method(draggingEntered:))]
        fn dragging_entered(&self, _sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(draggingUpdated:))]
        fn dragging_updated(&self, _sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(performDragOperation:))]
        fn perform_drag_operation(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> Bool {
            let paths = file_paths(sender);
            if paths.is_empty() {
                return Bool::NO;
            }
            crate::drop_on_icon(&self.ivars().app, paths);
            Bool::YES
        }
    }
);

impl DropView {
    fn new(mtm: MainThreadMarker, app: AppHandle) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars { app });
        unsafe { msg_send![super(this), init] }
    }
}

fn file_paths(info: &ProtocolObject<dyn NSDraggingInfo>) -> Vec<PathBuf> {
    let pasteboard = info.draggingPasteboard();
    let classes = NSArray::from_slice(&[NSURL::class()]);
    let Some(items) = (unsafe { pasteboard.readObjectsForClasses_options(&classes, None) }) else { return Vec::new() };
    items.iter().filter_map(|item| item.downcast::<NSURL>().ok()).filter_map(|url| url.to_file_path()).collect()
}

/// Put the drop view under the tray library's view in the status item's button.
pub fn accept_drops(item: Option<Retained<NSStatusItem>>, app: AppHandle) {
    let (Some(item), Some(mtm)) = (item, MainThreadMarker::new()) else { return };
    let Some(button) = item.button(mtm) else { return };
    let view = DropView::new(mtm, app);
    view.setFrame(button.bounds());
    view.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable);
    view.registerForDraggedTypes(&NSArray::from_slice(&[unsafe { NSPasteboardTypeFileURL }]));
    button.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Below, None);
}
