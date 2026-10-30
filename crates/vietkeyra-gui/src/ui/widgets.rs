use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, DropDown, Expander,
    Grid, HeaderBar, Label, Orientation, Separator, Switch,
};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use super::state::{GuiStateController, ShortcutOption, TypingMethod};
use crate::ipc_client::{self, IpcRequest};

pub struct GuiWidgets {
    pub window: ApplicationWindow,
    pub mode_switch: Switch,
    pub typing_dropdown: DropDown,
    pub shortcut_dropdown: DropDown,
    pub modern_tone_check: CheckButton,
    pub auto_undo_check: CheckButton,
    pub backspace_recovery_check: CheckButton,
    pub autostart_check: CheckButton,
    pub status_banner: GtkBox,
    pub banner_label: Label,
}

pub fn create_widgets(
    app: &Application,
    socket_path: PathBuf,
    state: GuiStateController,
    shutdown_tx: Sender<()>,
) -> GuiWidgets {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("VietKeyra - Bộ gõ Tiếng Việt cho Linux")
        .default_width(520)
        .default_height(680)
        .build();

    // -------------------------------------------------------------------------
    // HeaderBar
    // -------------------------------------------------------------------------
    let header_bar = HeaderBar::new();
    let title_box = GtkBox::new(Orientation::Vertical, 2);
    title_box.set_valign(Align::Center);

    let title_label = Label::builder()
        .label("<b>VietKeyra</b>")
        .use_markup(true)
        .build();
    let subtitle_label = Label::builder()
        .label("<small>Bộ gõ Tiếng Việt cho Linux</small>")
        .use_markup(true)
        .build();
    title_box.append(&title_label);
    title_box.append(&subtitle_label);
    header_bar.set_title_widget(Some(&title_box));
    window.set_titlebar(Some(&header_bar));

    // -------------------------------------------------------------------------
    // Main Content Box
    // -------------------------------------------------------------------------
    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(16);
    content_box.set_margin_bottom(16);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);

    // -------------------------------------------------------------------------
    // Status Banner (Daemon offline notice)
    // -------------------------------------------------------------------------
    let status_banner = GtkBox::new(Orientation::Horizontal, 8);
    status_banner.set_margin_bottom(4);
    let banner_label = Label::builder()
        .label("Không thể kết nối đến vietkeyra-daemon. Vui lòng kiểm tra dịch vụ.")
        .wrap(true)
        .build();
    status_banner.append(&banner_label);
    content_box.append(&status_banner);

    // -------------------------------------------------------------------------
    // Section 1: Trạng thái & Chế độ gõ
    // -------------------------------------------------------------------------
    let s1_label = Label::builder()
        .label("<b>Trạng thái &amp; Chế độ gõ</b>")
        .use_markup(true)
        .xalign(0.0)
        .build();
    content_box.append(&s1_label);

    let s1_box = GtkBox::new(Orientation::Vertical, 10);

    // Row 1: Mode Switch
    let switch_row = GtkBox::new(Orientation::Horizontal, 12);
    let switch_title = Label::builder()
        .label("Bật gõ tiếng Việt (Telex)")
        .hexpand(true)
        .xalign(0.0)
        .build();
    let mode_switch = Switch::new();
    mode_switch.set_active(state.get_settings().is_vietnamese);
    switch_row.append(&switch_title);
    switch_row.append(&mode_switch);
    s1_box.append(&switch_row);

    // Row 2: Typing Method DropDown
    let method_row = GtkBox::new(Orientation::Horizontal, 12);
    let method_title = Label::builder()
        .label("Kiểu gõ")
        .hexpand(true)
        .xalign(0.0)
        .build();
    let typing_dropdown = DropDown::from_strings(TypingMethod::all_labels());
    typing_dropdown.set_selected(state.get_settings().typing_method.to_index());
    method_row.append(&method_title);
    method_row.append(&typing_dropdown);
    s1_box.append(&method_row);

    content_box.append(&s1_box);
    content_box.append(&Separator::new(Orientation::Horizontal));

    // -------------------------------------------------------------------------
    // Section 2: Cấu hình Phím tắt Chuyển đổi
    // -------------------------------------------------------------------------
    let s2_label = Label::builder()
        .label("<b>Cấu hình Phím tắt Chuyển đổi</b>")
        .use_markup(true)
        .xalign(0.0)
        .build();
    content_box.append(&s2_label);

    let s2_box = GtkBox::new(Orientation::Vertical, 8);
    let shortcut_row = GtkBox::new(Orientation::Horizontal, 12);
    let shortcut_title = Label::builder()
        .label("Phím tắt chuyển đổi Tiếng Việt / Tiếng Anh")
        .hexpand(true)
        .xalign(0.0)
        .build();
    let shortcut_dropdown = DropDown::from_strings(ShortcutOption::all_labels());
    shortcut_dropdown.set_selected(state.get_settings().shortcut.to_index());
    shortcut_row.append(&shortcut_title);
    shortcut_row.append(&shortcut_dropdown);
    s2_box.append(&shortcut_row);

    content_box.append(&s2_box);
    content_box.append(&Separator::new(Orientation::Horizontal));

    // -------------------------------------------------------------------------
    // Section 3: Quy tắc Chính tả & Khởi động
    // -------------------------------------------------------------------------
    let s3_label = Label::builder()
        .label("<b>Quy tắc Chính tả &amp; Khởi động</b>")
        .use_markup(true)
        .xalign(0.0)
        .build();
    content_box.append(&s3_label);

    let s3_box = GtkBox::new(Orientation::Vertical, 6);
    let modern_tone_check =
        CheckButton::with_label("Đặt dấu thanh chuẩn mới (ví dụ: hòa, thúy thay vì hoà, thuý)");
    modern_tone_check.set_active(state.get_settings().modern_tone);

    let auto_undo_check =
        CheckButton::with_label("Tự động gỡ dấu khi gõ lặp phím (ví dụ: as -> á, ass -> as)");
    auto_undo_check.set_active(state.get_settings().auto_undo_repeat);

    let backspace_check =
        CheckButton::with_label("Cho phép gõ tắt và phục hồi từ khi nhấn Backspace");
    backspace_check.set_active(state.get_settings().backspace_recovery);

    let autostart_check = CheckButton::with_label("Tự động khởi động cùng Hyprland khi đăng nhập");
    autostart_check.set_active(state.get_settings().autostart_hyprland);

    s3_box.append(&modern_tone_check);
    s3_box.append(&auto_undo_check);
    s3_box.append(&backspace_check);
    s3_box.append(&autostart_check);

    content_box.append(&s3_box);
    content_box.append(&Separator::new(Orientation::Horizontal));

    // -------------------------------------------------------------------------
    // Section 4: Bảng tra cứu quy tắc gõ Telex (Expander)
    // -------------------------------------------------------------------------
    let expander = Expander::new(Some("Bảng tra cứu quy tắc gõ Telex"));
    let guide_grid = Grid::new();
    guide_grid.set_column_spacing(16);
    guide_grid.set_row_spacing(8);
    guide_grid.set_margin_top(8);
    guide_grid.set_margin_bottom(8);
    guide_grid.set_margin_start(8);

    // Guide Header
    let h1 = Label::builder()
        .label("<b>Quy tắc</b>")
        .use_markup(true)
        .xalign(0.0)
        .build();
    let h2 = Label::builder()
        .label("<b>Phím gõ</b>")
        .use_markup(true)
        .xalign(0.0)
        .build();
    let h3 = Label::builder()
        .label("<b>Kết quả</b>")
        .use_markup(true)
        .xalign(0.0)
        .build();
    guide_grid.attach(&h1, 0, 0, 1, 1);
    guide_grid.attach(&h2, 1, 0, 1, 1);
    guide_grid.attach(&h3, 2, 0, 1, 1);

    let guide_data = [
        ("Dấu Sắc", "s", "á, é, ó, ú, í, ý"),
        ("Dấu Huyền", "f", "à, è, ò, ù, ì, ỳ"),
        ("Dấu Hỏi", "r", "ả, ẻ, ỏ, ủ, ỉ, ỷ"),
        ("Dấu Ngã", "x", "ã, ẽ, õ, ũ, ĩ, ỹ"),
        ("Dấu Nặng", "j", "ạ, ẹ, ọ, ụ, ị, ỵ"),
        ("Mũ â / ê / ô", "aa / ee / oo", "â, ê, ô"),
        ("Móc ă / ơ / ư", "aw / ow / uw", "ă, ơ, ư"),
        ("Chữ Đ", "dd", "đ, Đ"),
        ("Phím w đơn", "w", "ư, Ư"),
        ("Gỡ dấu lặp", "as + s", "as (gỡ dấu sắc)"),
    ];

    for (row_idx, (rule, key, res)) in guide_data.iter().enumerate() {
        let r = (row_idx + 1) as i32;
        let l1 = Label::builder().label(*rule).xalign(0.0).build();
        let l2 = Label::builder()
            .label(format!("<code>{}</code>", key))
            .use_markup(true)
            .xalign(0.0)
            .build();
        let l3 = Label::builder()
            .label(format!("<b>{}</b>", res))
            .use_markup(true)
            .xalign(0.0)
            .build();

        guide_grid.attach(&l1, 0, r, 1, 1);
        guide_grid.attach(&l2, 1, r, 1, 1);
        guide_grid.attach(&l3, 2, r, 1, 1);
    }

    expander.set_child(Some(&guide_grid));
    content_box.append(&expander);
    content_box.append(&Separator::new(Orientation::Horizontal));

    // -------------------------------------------------------------------------
    // Section 5: Action Buttons (Bottom Bar)
    // -------------------------------------------------------------------------
    let action_box = GtkBox::new(Orientation::Horizontal, 12);
    action_box.set_halign(Align::End);
    action_box.set_margin_top(8);

    let restart_btn = Button::builder()
        .label("Khởi động lại Dịch vụ")
        .icon_name("view-refresh-symbolic")
        .build();

    let close_btn = Button::builder().label("Đóng").build();

    action_box.append(&restart_btn);
    action_box.append(&close_btn);
    content_box.append(&action_box);

    // -------------------------------------------------------------------------
    // Signal Connectors & Callbacks
    // -------------------------------------------------------------------------
    // 1. Switch toggle
    let sock_for_switch = socket_path.clone();
    let state_for_switch = state.clone();
    mode_switch.connect_state_set(move |_sw, active| {
        if state_for_switch.is_guard_active() {
            return glib::Propagation::Proceed;
        }

        let new_mode = if active { "VI" } else { "EN" };
        let req = IpcRequest::SetMode {
            mode: new_mode.to_string(),
        };
        let _ = ipc_client::send_command(&sock_for_switch, &req);

        glib::Propagation::Proceed
    });

    // 2. Typing method changed
    let sock_for_method = socket_path.clone();
    let state_for_method = state.clone();
    typing_dropdown.connect_selected_notify(move |dd| {
        if state_for_method.is_guard_active() {
            return;
        }
        let selected_idx = dd.selected();
        let method = TypingMethod::from_index(selected_idx);
        let mut cur = state_for_method.get_settings();
        cur.typing_method = method;
        *state_for_method.settings.lock().unwrap() = cur;

        let engine_str = match method {
            TypingMethod::BasicTelex => "telex",
            TypingMethod::ExpandedTelex => "telex_expanded",
            TypingMethod::Disabled => "none",
        };
        let req = IpcRequest::SetEngine {
            engine: engine_str.to_string(),
        };
        let _ = ipc_client::send_command(&sock_for_method, &req);
    });

    // 3. Restart service button
    let sock_for_restart = socket_path.clone();
    let state_for_restart = state.clone();
    let banner_label_for_restart = banner_label.clone();
    let banner_box_for_restart = status_banner.clone();
    let switch_for_restart = mode_switch.clone();

    restart_btn.connect_clicked(move |_| {
        let req = IpcRequest::ReloadConfig;
        match ipc_client::send_command(&sock_for_restart, &req) {
            Ok(resp) => {
                banner_box_for_restart.set_visible(false);
                if let Some(mode) = resp.mode() {
                    state_for_restart.set_guard(true);
                    switch_for_restart.set_active(mode.eq_ignore_ascii_case("VI"));
                    state_for_restart.set_guard(false);
                }
            }
            Err(e) => {
                banner_label_for_restart.set_text(&format!("Lỗi kết nối: {e}"));
                banner_box_for_restart.set_visible(true);
            }
        }
    });

    // 4. Close button
    let win_for_close = window.clone();
    close_btn.connect_clicked(move |_| {
        win_for_close.close();
    });

    // 5. Clean window close (stopping subscriber thread)
    window.connect_close_request(move |_| {
        let _ = shutdown_tx.send(());
        glib::Propagation::Proceed
    });

    // Initial check of daemon connectivity
    match ipc_client::send_command(&socket_path, &IpcRequest::GetStatus) {
        Ok(resp) => {
            status_banner.set_visible(false);
            if let Some(mode) = resp.mode() {
                state.set_guard(true);
                mode_switch.set_active(mode.eq_ignore_ascii_case("VI"));
                state.set_guard(false);
            }
        }
        Err(err) => {
            banner_label.set_text(&format!("Chế độ ngoại tuyến: {err}"));
            status_banner.set_visible(true);
        }
    }

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&content_box)
        .build();

    window.set_child(Some(&scrolled));

    GuiWidgets {
        window,
        mode_switch,
        typing_dropdown,
        shortcut_dropdown,
        modern_tone_check,
        auto_undo_check,
        backspace_recovery_check: backspace_check,
        autostart_check,
        status_banner,
        banner_label,
    }
}
