# 트레이 아이콘 위치 기반 윈도우 배치 구현

## 질문 내용

트레이 아이콘을 클릭하거나 트레이 메뉴의 "Open"을 선택했을 때, 윈도우가 트레이 아이콘의 실제 위치 바로 아래에 나타나도록 하는 방법

## 질문에 대한 이해

기존에는 윈도우 위치를 하드코딩된 좌표(`x: 100, y: 30`)로 설정했기 때문에, 트레이 아이콘의 실제 위치와 상관없이 항상 같은 위치에 윈도우가 나타났습니다. 트레이 아이콘의 실제 위치를 파악하고, 그 위치를 기준으로 윈도우를 배치해야 합니다.

## 답변

### 1. 핵심 개념: TrayIconEvent의 rect 필드

Tauri v2의 `TrayIconEvent`는 `rect` 필드를 제공합니다. 이 필드는 트레이 아이콘의 화면상 위치와 크기 정보를 담고 있습니다.

```rust
pub struct TrayIconEvent {
    pub rect: tauri::Rect,  // 트레이 아이콘의 위치와 크기
    // ... 기타 필드
}

pub struct Rect {
    pub position: PhysicalPosition<i32>,  // x, y 좌표
    pub size: PhysicalSize<u32>,          // width, height
}
```

### 2. 구현 단계

#### Step 1: AppState에 트레이 아이콘 위치 저장소 추가

트레이 아이콘의 위치는 메뉴 이벤트에서도 사용해야 하므로, 전역 상태로 저장합니다.

```rust
struct AppState {
    db: Mutex<ClipboardDatabase>,
    last_tray_rect: Mutex<Option<tauri::Rect>>,  // 트레이 아이콘 위치 저장
}
```

초기화:
```rust
.manage(AppState {
    db: Mutex::new(db),
    last_tray_rect: Mutex::new(None),
})
```

#### Step 2: on_tray_icon_event에서 rect 캡처 및 저장

트레이 아이콘 클릭 이벤트에서 `rect` 필드를 캡처하고 AppState에 저장합니다.

```rust
.on_tray_icon_event(|tray, event| {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        rect,  // 트레이 아이콘의 위치와 크기 정보
        ..
    } = event
    {
        let app = tray.app_handle();

        // 1. 트레이 아이콘 위치를 AppState에 저장
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(mut last_rect) = state.last_tray_rect.lock() {
                *last_rect = Some(rect);
            }
        }

        // 2. 윈도우 위치 계산 및 표시
        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                // 위치 계산 로직
                let window_width = 400;
                let window_x = rect.position.x + (rect.size.width as i32 / 2) - (window_width / 2);
                let window_y = rect.position.y + rect.size.height as i32 + 5;

                let _ = window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition { x: window_x, y: window_y }
                ));
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
})
```

#### Step 3: on_menu_event에서 저장된 rect 사용

메뉴 이벤트는 `rect` 정보를 직접 제공하지 않으므로, AppState에 저장된 마지막 위치를 사용합니다.

```rust
.on_menu_event(|app, event| {
    match event.id().as_ref() {
        "open" => {
            if let Some(window) = app.get_webview_window("main") {
                // AppState에서 저장된 트레이 위치 가져오기
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(last_rect) = state.last_tray_rect.lock() {
                        if let Some(rect) = *last_rect {
                            // 동일한 위치 계산 로직 적용
                            let window_width = 400;
                            let window_x = rect.position.x + (rect.size.width as i32 / 2) - (window_width / 2);
                            let window_y = rect.position.y + rect.size.height as i32 + 5;

                            let _ = window.set_position(tauri::Position::Physical(
                                tauri::PhysicalPosition { x: window_x, y: window_y }
                            ));
                        }
                    }
                }
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        // ... 기타 메뉴 항목
    }
})
```

### 3. 위치 계산 로직 설명

```rust
let window_width = 400;  // tauri.conf.json에 정의된 윈도우 너비

// X 좌표: 트레이 아이콘 중앙 아래에 윈도우를 중앙 정렬
let window_x = rect.position.x          // 트레이 아이콘 좌측 x 좌표
             + (rect.size.width as i32 / 2)  // + 트레이 아이콘 너비의 절반 = 중앙
             - (window_width / 2);           // - 윈도우 너비의 절반 = 중앙 정렬

// Y 좌표: 트레이 아이콘 바로 아래 + 약간의 여백
let window_y = rect.position.y          // 트레이 아이콘 상단 y 좌표
             + rect.size.height as i32  // + 트레이 아이콘 높이 = 하단
             + 5;                       // + 5px 여백
```

**시각적 표현:**
```
┌─────────────────┐
│    Menu Bar     │
│  [🔭]           │ ← 트레이 아이콘 (rect.position.x, rect.position.y)
└─────────────────┘   (rect.size.width, rect.size.height)
       ↓ +5px
  ┌─────────┐
  │ Window  │ ← 중앙 정렬된 윈도우
  │         │
  └─────────┘
```

### 4. 주요 포인트

1. **rect 필드 캡처**: `TrayIconEvent::Click` 패턴 매칭에서 `rect` 필드를 명시적으로 추출
2. **상태 저장**: 메뉴 이벤트에서도 사용할 수 있도록 AppState에 저장
3. **중앙 정렬**: 트레이 아이콘 중앙 아래에 윈도우를 배치하여 자연스러운 UX 제공
4. **여백 추가**: `+5px` 여백으로 트레이 아이콘과 윈도우 사이에 시각적 공간 확보

### 5. 주의사항

- **윈도우 크기 동기화**: `window_width` 값은 `tauri.conf.json`의 `width` 설정과 일치해야 함
- **멀티 모니터**: 트레이 아이콘과 윈도우가 다른 모니터에 배치되는 경우는 고려되지 않음 (일반적으로 메뉴바는 주 모니터에만 존재)
- **화면 경계**: 화면 가장자리에 트레이 아이콘이 있을 경우 윈도우가 화면 밖으로 나갈 수 있음 (필요시 boundary check 추가)

### 6. 파일 위치

- **구현 파일**: `src-tauri/src/lib.rs`
  - AppState 정의: lines 18-21
  - on_tray_icon_event: lines 188-223
  - on_menu_event: lines 152-187
