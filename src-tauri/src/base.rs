use objc2_app_kit::NSPasteboard;

pub struct Watcher {
    count: isize
}

impl Watcher {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn get_clipboard_text(&mut self) -> Option<String> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let count = pasteboard.changeCount();
        if self.count == count {
            return None;
        }
        self.count = count;
        let type_string = unsafe { objc2_app_kit::NSPasteboardTypeString };
        if let Some(string) = pasteboard.stringForType(type_string) {
            Some(string.to_string())
        } else {
            None
        }
    }
}

pub fn get_clipboard_text() -> String {
    let pasteboard = NSPasteboard::generalPasteboard();
    let count = pasteboard.changeCount();
    let type_string = unsafe { objc2_app_kit::NSPasteboardTypeString };
    if let Some(string) = pasteboard.stringForType(type_string) {
        string.to_string()
    } else {
        "".to_string()
    }
}