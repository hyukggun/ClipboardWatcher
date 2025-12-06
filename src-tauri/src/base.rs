use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};

pub fn get_clipboard_text() -> String {
    let pasteboard = NSPasteboard::generalPasteboard();
    let type_string = unsafe { objc2_app_kit::NSPasteboardTypeString };
    if let Some(string) = pasteboard.stringForType(type_string) {
        string.to_string()
    } else {
        "".to_string()
    }
}