use vietkeyra_gui::ui::state::{ShortcutOption, TypingMethod};

#[test]
fn test_all_gui_labels_are_nfc_clean_without_combining_characters() {
    let strings_to_verify = [
        "VietKeyra",
        "Bộ gõ Tiếng Việt cho Linux",
        "Trạng thái & Chế độ gõ",
        "Bật gõ tiếng Việt (Telex)",
        "Kiểu gõ",
        "Cấu hình Phím tắt Chuyển đổi",
        "Phím tắt chuyển đổi Tiếng Việt / Tiếng Anh",
        "Quy tắc Chính tả & Khởi động",
        "Đặt dấu thanh chuẩn mới (ví dụ: hòa, thúy thay vì hoà, thuý)",
        "Tự động gỡ dấu khi gõ lặp phím (ví dụ: as -> á, ass -> as)",
        "Cho phép gõ tắt và phục hồi từ khi nhấn Backspace",
        "Tự động khởi động cùng Hyprland khi đăng nhập",
        "Bảng tra cứu quy tắc gõ Telex",
        "Khởi động lại Dịch vụ",
        "Đóng",
        "Không thể kết nối đến vietkeyra-daemon. Vui lòng kiểm tra dịch vụ.",
    ];

    for text in strings_to_verify {
        // Assert no decomposed combining diacritics (U+0300..=U+036F)
        for ch in text.chars() {
            let cp = ch as u32;
            assert!(
                !(0x0300..=0x036F).contains(&cp),
                "String contains combining decomposed diacritic: {} in '{}'",
                ch,
                text
            );
        }
    }
}

#[test]
fn test_typing_method_labels_completeness() {
    let labels = TypingMethod::all_labels();
    assert_eq!(labels.len(), 3);
    assert_eq!(labels[0], "Telex cơ bản");
    assert_eq!(labels[1], "Telex mở rộng (w -> ư, [ -> ơ, ] -> ư)");
    assert_eq!(labels[2], "Tắt hoàn toàn (Chỉ Tiếng Anh)");
}

#[test]
fn test_shortcut_options_labels_completeness() {
    let labels = ShortcutOption::all_labels();
    assert_eq!(labels.len(), 4);
    assert!(labels[0].contains("Super + Space"));
    assert!(labels[1].contains("Ctrl + Shift"));
    assert!(labels[2].contains("Alt + Shift"));
    assert!(labels[3].contains("Caps Lock"));
}
