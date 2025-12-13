use objc2_app_kit::NSPasteboard;

pub fn get_current_clipboard_count() -> isize {
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.changeCount()
}

pub fn get_clipboard_image(_count: isize) -> Option<String>{
    use base64::{Engine as _, engine::general_purpose};

    let pasteboard = NSPasteboard::generalPasteboard();
    let image_type = unsafe { objc2_app_kit::NSPasteboardTypePNG };

    if let Some(data) = pasteboard.dataForType(image_type) {
        let data = data.to_vec();

        // Base64로 인코딩하여 data URL 반환
        let base64_image = general_purpose::STANDARD.encode(&data);
        let data_url = format!("data:image/png;base64,{}", base64_image);

        return Some(data_url);
    }
    None
}

pub fn get_clipboard_text() -> Option<String> {
    let pasteboard = NSPasteboard::generalPasteboard();
    let type_string = unsafe { objc2_app_kit::NSPasteboardTypeString };
    if let Some(string) = pasteboard.stringForType(type_string) {
        Some(string.to_string())
    } else {
        None
    }
}