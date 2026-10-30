# VietKeyra

**VietKeyra** là ứng dụng bộ gõ tiếng Việt độc lập hoàn toàn dành cho Linux, được thiết kế và tối ưu riêng cho môi trường **Arch Linux**, **Hyprland** trên **Wayland**, và **Waybar**.

Dự án tự xử lý 100% thuật toán Telex bằng ngôn ngữ Rust, **tuyệt đối không sử dụng, cài đặt, gọi lệnh hay phụ thuộc vào Fcitx5, IBus hoặc bất kỳ bộ gõ nào khác**.

---

## Tính Năng Nổi Bật

- **Engine Telex độc lập 100%**: Xử lý biến âm, dấu thanh, gỡ dấu khi lặp phím (`ss` -> `s`, `aa` -> `a`, `ww` -> `w`) và chuẩn hóa Unicode NFC trực tiếp với độ trễ dưới 1 microsecond.
- **Chuẩn đặt dấu truyền thống cố định**: Đặt dấu thanh chuẩn truyền thống (ví dụ: `hòa`, `thủy`, `khỏe`, `toán`).
- **Tự động kích hoạt**: Khi khởi động ứng dụng là tự động bật chế độ gõ Tiếng Việt (VI) ngay lập tức, không cần thao tác phụ.
- **Tích hợp Waybar mượt mà**: Hiển thị trạng thái `VI` hoặc `EN` trực tiếp trên thanh Waybar. Nhấp chuột vào biểu tượng để chuyển đổi luân phiên giữa Tiếng Anh và Tiếng Việt.
- **Giao thức Wayland Native**: Kết nối trực tiếp với Hyprland qua `zwp_input_method_v2` và `zwp_virtual_keyboard_v1`. Chạy hoàn toàn với quyền người dùng thông thường, **không đòi hỏi quyền root** hay udev rule đặc biệt.
- **Cài đặt & Gỡ bỏ tối giản**: Chỉ cần 1 lệnh cài đặt (`./install.sh`) và 1 lệnh gỡ sạch sẽ (`./uninstall.sh`).

---

## Kiến Trúc Hệ Thống

VietKeyra bao gồm 3 crate Rust trong Cargo Workspace:
- `vietkeyra-engine`: Lõi xử lý âm tiết tiếng Việt, bảng tra cứu NFC O(1) không cấp phát heap, thuật toán Telex và đặt dấu truyền thống.
- `vietkeyra-daemon` (`vietkeyrad`): Tiến trình daemon chạy nền kết nối Wayland compositor và quản lý Unix socket IPC (`/run/user/1000/vietkeyra.sock`).
- `vietkeyra-cli` (`vietkeyra-ctl`): Công cụ dòng lệnh giao tiếp với daemon và phát tín hiệu `RTMIN+8` tới Waybar.

---

## Hướng Dẫn Cài Đặt & Sử Dụng

### 1. Cài đặt tự động
Trong thư mục dự án, chạy lệnh:
```bash
./install.sh
```
Kịch bản sẽ tự động:
1. Biên dịch tối ưu phiên bản release bằng Rust.
2. Sao chép `vietkeyrad` và `vietkeyra-ctl` vào `~/.local/bin/`.
3. Tích hợp module hiển thị trạng thái `custom/vietkeyra` vào Waybar.
4. Cấu hình tự khởi động cùng phiên Hyprland (`Startup_Apps.conf`).
5. Khởi động ngay daemon `vietkeyrad` và làm mới Waybar.

### 2. Sử dụng
- Gõ văn bản tiếng Việt theo kiểu Telex chuẩn trong bất kỳ ứng dụng Wayland nào (Chrome, Mousepad, VS Code, v.v.).
- **Chuyển đổi Tiếng Việt / Tiếng Anh**: Nhấp chuột trái vào chữ **VI** trên thanh Waybar để đổi sang **EN** (hoặc ngược lại).
- Lệnh điều khiển thủ công từ terminal:
  ```bash
  vietkeyra-ctl waybar     # Xuất JSON cho Waybar
  vietkeyra-ctl toggle     # Đổi luân phiên EN / VI
  vietkeyra-ctl status     # Xem trạng thái hiện tại
  vietkeyra-ctl set-mode VI # Đặt chế độ cụ thể
  ```

### 3. Gỡ cài đặt
Để gỡ bỏ hoàn toàn VietKeyra và hoàn tác cấu hình:
```bash
./uninstall.sh
```

---

## Giấy Phép (License)
Phát hành theo giấy phép MIT License.
