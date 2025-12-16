# Tauri에서 새로운 윈도우 열기

## 질문 내용
Tauri v2 애플리케이션에서 새로운 윈도우를 프로그래밍 방식으로 여는 방법

## 질문에 대한 이해

Tauri는 다중 윈도우를 지원하며, 런타임에 동적으로 새로운 윈도우를 생성할 수 있습니다. 주요 사용 사례:
- Settings 윈도우
- About 윈도우
- 팝업 또는 모달 다이얼로그
- 멀티 모니터 지원을 위한 별도 윈도우

## 답변

### 방법 1: WebviewWindowBuilder 사용 (권장)

Rust 백엔드에서 `WebviewWindowBuilder`를 사용하여 새 윈도우를 생성합니다.

#### 기본 예제

```rust
use tauri::{Manager, WebviewWindowBuilder, WebviewUrl};

#[tauri::command]
fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    // 윈도우가 이미 존재하는지 확인
    if let Some(window) = app.get_webview_window("settings") {
        // 이미 존재하면 포커스만 이동
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    // 새 윈도우 생성
    WebviewWindowBuilder::new(
        &app,
        "settings",  // 윈도우 ID (고유해야 함)
        WebviewUrl::App("/settings".into())  // 라우트 경로
    )
    .title("Settings")
    .inner_size(600.0, 400.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

#### 현재 프로젝트의 실제 구현 (lib.rs:157-172)

```rust
"settings" => {
    // Check if settings window already exists
    if let Some(settings_window) = app.get_webview_window("settings") {
        let _ = settings_window.show();
        let _ = settings_window.set_focus();
    } else {
        // Create new settings window
        use tauri::WebviewWindowBuilder;
        let _ = WebviewWindowBuilder::new(
            app,
            "settings",
            tauri::WebviewUrl::App("/settings".into())
        )
        .title("Settings")
        .inner_size(600.0, 400.0)
        .resizable(true)
        .build();
    }
}
```

### WebviewWindowBuilder 주요 메서드

#### 필수 매개변수
```rust
WebviewWindowBuilder::new(
    app_handle,      // AppHandle 또는 &App
    "window_id",     // 고유 윈도우 ID
    WebviewUrl::App("/route".into())  // 앱 내부 라우트
)
```

#### 선택적 설정 메서드

| 메서드 | 설명 | 예제 |
|--------|------|------|
| `.title()` | 윈도우 타이틀 | `.title("Settings")` |
| `.inner_size()` | 윈도우 크기 (width, height) | `.inner_size(800.0, 600.0)` |
| `.min_inner_size()` | 최소 크기 | `.min_inner_size(400.0, 300.0)` |
| `.max_inner_size()` | 최대 크기 | `.max_inner_size(1920.0, 1080.0)` |
| `.resizable()` | 크기 조절 가능 여부 | `.resizable(true)` |
| `.minimizable()` | 최소화 가능 여부 | `.minimizable(true)` |
| `.maximizable()` | 최대화 가능 여부 | `.maximizable(true)` |
| `.closable()` | 닫기 가능 여부 | `.closable(true)` |
| `.decorations()` | 타이틀바 표시 여부 | `.decorations(true)` |
| `.always_on_top()` | 항상 위에 표시 | `.always_on_top(true)` |
| `.center()` | 화면 중앙에 배치 | `.center()` |
| `.position()` | 윈도우 위치 (x, y) | `.position(100.0, 100.0)` |
| `.fullscreen()` | 전체화면 | `.fullscreen(true)` |
| `.focused()` | 생성 시 포커스 | `.focused(true)` |
| `.visible()` | 생성 시 표시 여부 | `.visible(false)` |
| `.transparent()` | 투명 배경 | `.transparent(true)` |

### 고급 예제

#### 예제 1: 모달 다이얼로그 스타일 윈도우

```rust
#[tauri::command]
fn open_modal_dialog(app: tauri::AppHandle) -> Result<(), String> {
    WebviewWindowBuilder::new(
        &app,
        "modal_dialog",
        WebviewUrl::App("/dialog".into())
    )
    .title("Confirm Action")
    .inner_size(400.0, 200.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .always_on_top(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

#### 예제 2: 투명한 오버레이 윈도우

```rust
#[tauri::command]
fn open_overlay_window(app: tauri::AppHandle) -> Result<(), String> {
    WebviewWindowBuilder::new(
        &app,
        "overlay",
        WebviewUrl::App("/overlay".into())
    )
    .title("")
    .inner_size(300.0, 150.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .position(100.0, 100.0)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

#### 예제 3: 외부 URL 열기

```rust
#[tauri::command]
fn open_external_url(app: tauri::AppHandle) -> Result<(), String> {
    WebviewWindowBuilder::new(
        &app,
        "external_browser",
        WebviewUrl::External("https://github.com".parse().unwrap())
    )
    .title("GitHub")
    .inner_size(1024.0, 768.0)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

### 윈도우 관리

#### 기존 윈도우 확인 및 제어

```rust
// 윈도우 존재 여부 확인
if let Some(window) = app.get_webview_window("window_id") {
    // 윈도우 표시
    window.show().unwrap();

    // 윈도우 숨기기
    window.hide().unwrap();

    // 포커스 이동
    window.set_focus().unwrap();

    // 윈도우 닫기
    window.close().unwrap();

    // 윈도우 최소화
    window.minimize().unwrap();

    // 윈도우 최대화
    window.maximize().unwrap();
}
```

#### 윈도우 리스트 가져오기

```rust
use tauri::Manager;

#[tauri::command]
fn get_all_windows(app: tauri::AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .map(|k| k.to_string())
        .collect()
}
```

### 프론트엔드에서 호출

#### React/TypeScript에서 새 윈도우 열기

```typescript
import { invoke } from '@tauri-apps/api/core';

// 버튼 클릭 시 settings 윈도우 열기
async function openSettings() {
  try {
    await invoke('open_settings_window');
    console.log('Settings window opened');
  } catch (error) {
    console.error('Failed to open settings:', error);
  }
}

// JSX
<button onClick={openSettings}>
  Open Settings
</button>
```

### tauri.conf.json 설정

초기 윈도우는 `tauri.conf.json`에서 설정할 수 있습니다:

```json
{
  "app": {
    "windows": [
      {
        "title": "ClipboardWatcher",
        "width": 800,
        "height": 600,
        "resizable": true,
        "fullscreen": false,
        "center": true
      }
    ]
  }
}
```

### 베스트 프랙티스

#### 1. 윈도우 ID 중복 방지

```rust
// ❌ 나쁜 예: 중복 체크 없이 윈도우 생성
WebviewWindowBuilder::new(&app, "settings", url).build()?;

// ✅ 좋은 예: 중복 체크 후 생성
if app.get_webview_window("settings").is_none() {
    WebviewWindowBuilder::new(&app, "settings", url).build()?;
}
```

#### 2. 에러 처리

```rust
// ❌ 나쁜 예: 에러 무시
let _ = WebviewWindowBuilder::new(&app, "window", url).build();

// ✅ 좋은 예: 적절한 에러 처리
WebviewWindowBuilder::new(&app, "window", url)
    .build()
    .map_err(|e| {
        eprintln!("Failed to create window: {}", e);
        format!("Window creation failed: {}", e)
    })?;
```

#### 3. 윈도우 정리

```rust
// 윈도우를 명시적으로 닫기
#[tauri::command]
fn close_window(window: tauri::Window) {
    window.close().unwrap();
}
```

### 멀티 윈도우 통신

#### 윈도우 간 이벤트 전송

```rust
use tauri::Emitter;

#[tauri::command]
fn send_to_window(app: tauri::AppHandle, window_id: &str, message: String) {
    if let Some(window) = app.get_webview_window(window_id) {
        window.emit("message", message).unwrap();
    }
}
```

#### 프론트엔드에서 이벤트 수신

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<string>('message', (event) => {
  console.log('Received message:', event.payload);
});

// Cleanup
unlisten();
```

### 주의사항

1. **윈도우 ID는 고유해야 함**: 동일한 ID로 여러 윈도우를 생성할 수 없습니다
2. **라우팅 설정**: React Router 등을 사용하는 경우 해당 라우트가 존재해야 합니다
3. **메모리 관리**: 사용하지 않는 윈도우는 명시적으로 닫아야 합니다
4. **플랫폼 차이**: macOS, Windows, Linux에서 일부 설정의 동작이 다를 수 있습니다

### 디버깅 팁

```rust
// 윈도우 생성 로깅
match WebviewWindowBuilder::new(&app, "test", url).build() {
    Ok(window) => {
        println!("Window created successfully: {}", window.label());
    }
    Err(e) => {
        eprintln!("Failed to create window: {:?}", e);
    }
}
```

### 참고 자료

- [Tauri v2 Window API Documentation](https://v2.tauri.app/reference/javascript/window/)
- [Tauri WebviewWindowBuilder](https://docs.rs/tauri/2.0.0/tauri/webview/struct.WebviewWindowBuilder.html)
- [Multi-Window Example](https://github.com/tauri-apps/tauri/tree/dev/examples/multiwindow)

## 결론

Tauri v2에서 새 윈도우를 여는 것은 `WebviewWindowBuilder`를 사용하면 간단합니다. 중요한 것은:
1. 윈도우 ID의 고유성 보장
2. 기존 윈도우 존재 여부 확인
3. 적절한 에러 처리
4. 플랫폼별 차이 고려

현재 프로젝트의 settings 윈도우 구현이 좋은 참고 예제입니다.
