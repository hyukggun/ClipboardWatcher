# ClipboardWatcher 개발 Q&A

> 프로젝트 개발 중 발생한 질문과 해결 방법을 정리한 문서입니다.

---

## 목차

1. [SQLite 데이터베이스 파일 생성](#1-sqlite-데이터베이스-파일-생성)
2. [클립보드 폴링을 위한 타이머 구현](#2-클립보드-폴링을-위한-타이머-구현)
3. [폴링 스레드가 실행되지 않는 문제](#3-폴링-스레드가-실행되지-않는-문제)
4. [Tauri 백엔드에서 프론트엔드로 데이터 전송](#4-tauri-백엔드에서-프론트엔드로-데이터-전송)
5. [Svelte 환경 변수 import 오류](#5-svelte-환경-변수-import-오류)
6. [Svelte 콘솔 로그 확인 방법](#6-svelte-콘솔-로그-확인-방법)

---

## 1. SQLite 데이터베이스 파일 생성

### 질문
> SQLite 데이터베이스 파일을 생성하기 위해서는 어떻게 해야 하나?

### 질문에 대한 이해

프로젝트에서 클립보드 히스토리를 저장하기 위해 SQLite 데이터베이스를 사용해야 합니다. Rust에서 SQLite 파일을 생성하고 초기화하는 방법이 필요했습니다.

### 답변

#### 현재 구현 상태

프로젝트에는 이미 SQLite 관련 코드가 구현되어 있습니다:

**Cargo.toml**:
```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
chrono = "0.4"
```

**db.rs**:
- `ClipboardDatabase::new(db_path)`: 데이터베이스 파일 생성 및 연결
- `init_schema()`: 테이블과 인덱스 자동 생성
- `Connection::open(db_path)`: 파일이 없으면 자동으로 생성

#### 핵심 개념

1. **자동 생성**: rusqlite의 `Connection::open()`은 파일이 존재하지 않으면 자동으로 생성
2. **멱등성**: `CREATE TABLE IF NOT EXISTS`는 여러 번 실행해도 안전
3. **초기화**: `init_schema()`가 테이블 구조를 설정

#### 개선이 필요한 부분

**lib.rs:56-66**의 데이터베이스 경로 설정:

```rust
// 현재: 현재 디렉토리에 저장 (문제 있음)
let db_path = std::path::PathBuf::from("clipboard_history.db");
```

**권장 사항**:
- Tauri의 `app.path()` API 사용
- 플랫폼별 적절한 앱 데이터 디렉토리 사용
  - macOS: `~/Library/Application Support/[app-name]/`
  - Windows: `C:\Users\[user]\AppData\Local\[app-name]\`
  - Linux: `~/.local/share/[app-name]/`

**구현 힌트**:
```rust
// setup 훅에서 앱 핸들 사용
.setup(|app| {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("clipboard_history.db");

    // 데이터베이스 초기화
    Ok(())
})
```

---

## 2. 클립보드 폴링을 위한 타이머 구현

### 질문
> 현재는 `get_clipboard_text`를 호출해야만 복사한 텍스트를 가져올 수 있어. 폴링 방식 + 타이머를 통해 일정 간격으로 계속해서 텍스트를 가져오도록 하려고 해. 타이머를 사용하는 방법을 알려줘.

### 질문에 대한 이해

**현재 구조의 문제점**:
- 수동 호출 방식: 프론트엔드에서 명시적으로 호출해야 함
- 실시간성 부족: 클립보드 변경을 즉시 감지하지 못함

**원하는 구조**:
- 자동 폴링: 백그라운드에서 일정 간격으로 확인
- 변경 감지: 클립보드 내용이 변경되면 자동으로 데이터베이스에 저장
- 비동기 처리: 메인 스레드를 블로킹하지 않음

### 답변

상세한 가이드는 [`clipboard-polling-timer-guide.md`](./clipboard-polling-timer-guide.md)를 참조하세요.

#### 구현 방법 요약

**Option A: `std::thread` + `std::thread::sleep`**
- 장점: 표준 라이브러리만 사용, 간단
- 단점: 타이머 취소 어려움, 유연성 부족

**Option B: `tokio` + `tokio::time::interval`** (권장)
- 장점: 비동기 처리, Tauri와 잘 통합됨
- 단점: tokio 런타임 필요

**Option C: Tauri Event System**
- 장점: Tauri 생태계와 완벽한 통합
- 단점: Tauri 특화

#### 핵심 구현 포인트

1. **폴링 간격**: 1-2초 권장 (반응성과 효율성의 균형)
2. **중복 감지**: 이전 클립보드 내용과 비교하여 변경된 경우만 저장
3. **앱 상태 통합**: `AppState`의 데이터베이스에 접근
4. **에러 처리**: 클립보드 접근 실패, DB 저장 실패 처리

#### 의사코드

```rust
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                monitor_clipboard(app_handle).await;
            });
            Ok(())
        })
}

async fn monitor_clipboard(app_handle: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut last_content = String::new();

    loop {
        interval.tick().await;
        // 클립보드 확인 및 변경 감지
        // 변경되었으면 저장 및 이벤트 발생
    }
}
```

---

## 3. 폴링 스레드가 실행되지 않는 문제

### 질문
> `main.rs`에서 `polling_clipboard`를 실행했을 때 반복적으로 "Polling clipboard"가 호출되어야 하는데 호출이 되지 않아. 원인을 알 수 있을까?

### 질문에 대한 이해

**구현된 코드**:
```rust
fn polling_clipboard(interval: Duration) {
    loop {
        let text = base::get_clipboard_text();
        println!("Polling clipboard: {}", text);
        thread::sleep(interval);
    }
}

fn main() {
    thread::spawn(|| {
        println!("Starting clipboard polling");
        polling_clipboard(Duration::from_secs(1));
    });
    clipboardwatcher_lib::run()
}
```

**예상되는 문제**: 반복문이 실행되지 않음

### 답변

#### 원인: macOS Cocoa API의 스레드 제약사항

**핵심 문제**:
- `base::get_clipboard_text()`는 `NSPasteboard::generalPasteboard()`를 사용
- macOS의 Cocoa 프레임워크(NSPasteboard 포함)는 **메인 스레드에서만 안전하게 동작**
- `thread::spawn()`으로 생성된 백그라운드 스레드는 메인 스레드가 아님
- 백그라운드 스레드에서 호출 시 패닉, 조용한 실패, 또는 정의되지 않은 동작 발생

#### 디버깅 방법

**1단계: 패닉 캐치**
```rust
thread::spawn(|| {
    println!("Starting clipboard polling");

    std::panic::catch_unwind(|| {
        polling_clipboard(Duration::from_secs(1));
    }).unwrap_or_else(|err| {
        eprintln!("Thread panicked: {:?}", err);
    });
});
```

**2단계: 로그로 원인 분리**
```rust
fn polling_clipboard(interval: Duration) {
    loop {
        println!("Before get_clipboard_text");
        let text = base::get_clipboard_text();
        println!("After get_clipboard_text: {}", text);
        thread::sleep(interval);
    }
}
```

#### 해결 방향

**Option A: macOS 메인 런루프 통합**
- `dispatch_async` on main queue 사용
- 복잡하지만 Cocoa API와 호환

**Option B: Tauri 비동기 시스템 활용** (권장)
- Tauri의 setup hook과 tokio 사용
- 메인 스레드에서 실행되는 환경 활용

**Option C: 크로스 플랫폼 클립보드 라이브러리**
- `clipboard` 또는 `arboard` 크레이트 사용
- 스레드 안전성이 보장됨
- macOS, Windows, Linux 지원

**Cargo.toml에 추가**:
```toml
[dependencies]
arboard = "3.4"
```

**사용 예시**:
```rust
use arboard::Clipboard;

fn get_clipboard_text() -> String {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.get_text().unwrap_or_default()
}
```

---

## 4. Tauri 백엔드에서 프론트엔드로 데이터 전송

### 질문
> 폴링을 사용하면 일반적으로 프론트엔드 → 백엔드로 요청을 보내는 것과 달리, 백엔드에서 프론트로 보내야 하는데 Tauri에서는 어떤 식으로 프론트로 데이터를 보낼 수 있어?

### 질문에 대한 이해

**통신 패턴의 차이**:
- **전통적인 방식**: 프론트엔드 `invoke()` → 백엔드 `#[tauri::command]` → 응답
- **폴링 시나리오**: 백엔드에서 변경 감지 → 프론트엔드에 알림 (역방향)

### 답변

#### Tauri Event System

Tauri는 **Event-Driven Architecture**를 제공합니다.

**핵심 개념**:
- 백엔드: 이벤트 발생 (emit)
- 프론트엔드: 이벤트 리스닝 (listen)
- 느슨한 결합, 실시간 업데이트 가능

#### 세 가지 이벤트 방법

**1. Window-Specific Events**
```rust
// 백엔드
window.emit("event-name", payload)?;

// 프론트엔드
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('event-name', (event) => {
  console.log(event.payload);
});
```

**2. Global Events (모든 윈도우)**
```rust
// 백엔드
app_handle.emit_all("event-name", payload)?;
```

**3. Targeted Events (특정 윈도우)**
```rust
// 백엔드
if let Some(window) = app_handle.get_window("main") {
    window.emit("event-name", payload)?;
}
```

#### 클립보드 모니터링 적용 예시

**백엔드 (lib.rs)**:
```rust
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            spawn_clipboard_monitor(app_handle);
            Ok(())
        })
}

fn spawn_clipboard_monitor(app_handle: AppHandle) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut last_content = String::new();

        loop {
            interval.tick().await;

            // 클립보드 확인
            let current_content = get_clipboard_text();

            // 변경 감지
            if current_content != last_content && !current_content.is_empty() {
                last_content = current_content.clone();

                // 데이터베이스에 저장
                // ...

                // 프론트엔드에 이벤트 발생
                let payload = ClipboardEntry {
                    id: Some(id),
                    content: current_content,
                    created_at: timestamp,
                };

                app_handle.emit_all("clipboard-changed", payload).ok();
            }
        }
    });
}
```

**프론트엔드 (+page.svelte)**:
```javascript
<script>
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let clipboardHistory = $state([]);
  let unlisten;

  onMount(async () => {
    unlisten = await listen('clipboard-changed', (event) => {
      const entry = event.payload;
      clipboardHistory = [entry, ...clipboardHistory];
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>
```

#### 중요한 고려사항

1. **이벤트 이름 규칙**: 케밥 케이스 사용 (`clipboard-changed`)
2. **에러 처리**: `emit()`은 `Result` 반환, `.ok()`로 무시 가능
3. **성능**: 너무 빈번한 이벤트는 성능 저하, 디바운싱 고려
4. **타입 안전성**: 백엔드는 `Serialize`, 프론트엔드는 TypeScript 인터페이스

---

## 5. Svelte 환경 변수 import 오류

### 질문
> `spawn_clipboard_polling_thread`를 추가했는데, Svelte에서 "Cannot import $env/static/private into code that runs in the browser, as this could leak sensitive information." 오류가 발생하는데 원인을 알 수 있을까?

### 질문에 대한 이해

백엔드 코드를 추가한 후 프론트엔드에서 환경 변수 관련 오류가 발생했습니다.

### 답변

#### 원인

**잘못된 import (src/routes/+page.svelte:2)**:
```javascript
import { _ } from "$env/static/private";
```

이 import는 코드에서 전혀 사용되지 않으며, SPA 모드에서 사용할 수 없습니다.

#### 왜 에러가 발생하는가?

**SvelteKit의 환경 변수 시스템**:
- `$env/static/private`: 서버 사이드 전용 (API 키, DB 비밀번호)
- `$env/static/public`: 클라이언트 사이드 가능 (공개 설정)

**Tauri + SvelteKit SPA 모드**:
- Tauri는 SPA (Single Page Application) 모드로 실행
- SPA 모드 = SSR 없음 = 모든 코드가 브라우저에서 실행
- `$env/static/private`를 사용하면 민감한 정보가 브라우저에 노출됨
- SvelteKit이 이를 방지하기 위해 컴파일 에러 발생

#### 해결 방법

**Option 1: Import 제거** (권장)
```diff
<script lang="ts">
-  import { _ } from "$env/static/private";
   import { invoke } from "@tauri-apps/api/core";
   // ...
```

**Option 2: Public 환경 변수로 변경**
```javascript
import { PUBLIC_SOME_VAR } from "$env/static/public";
```
(이름이 `PUBLIC_`로 시작해야 함)

**Option 3: 백엔드에서 관리** (민감한 정보)
```rust
#[tauri::command]
fn get_config() -> Config {
    // 환경 변수나 설정 파일에서 읽기
    Config { /* ... */ }
}
```

#### Tauri 설정 관리 Best Practice

**프론트엔드 (공개 정보)**:
- 테마 설정
- API 엔드포인트 (공개)
- UI 기본값

**백엔드 (민감한 정보)**:
- API 키
- 데이터베이스 연결 정보
- 암호화 키
- 파일 경로

---

## 6. Svelte 콘솔 로그 확인 방법

### 질문
> Svelte에서의 콘솔 로그는 어디서 확인 가능해?

### 질문에 대한 이해

Svelte 코드의 `console.log()` 출력을 어디서 볼 수 있는지에 대한 질문입니다.

### 답변

#### 핵심 개념

**Tauri 앱의 두 가지 실행 환경**:
- **백엔드 (Rust)**: 터미널에 로그 출력 (`println!`, `eprintln!`)
- **프론트엔드 (Svelte/JS)**: 브라우저 DevTools에 로그 출력 (`console.log`)

**웹뷰 = 내장 브라우저**:
- Tauri 앱의 프론트엔드는 웹뷰(WebView)에서 실행
- macOS: Safari WebKit 기반
- 일반 웹 앱처럼 DevTools 사용 가능

#### DevTools 여는 방법

**키보드 단축키**:
- **macOS**: `Cmd + Option + I`
- **Windows/Linux**: `Ctrl + Shift + I` 또는 `F12`

**마우스**: 앱 내에서 우클릭 → "Inspect Element" 또는 "검사"

#### DevTools에서 확인할 수 있는 것

**Console 탭**:
- `console.log()`, `console.error()`, `console.warn()` 출력
- JavaScript 런타임 에러

**Elements 탭**:
- HTML 구조 확인
- CSS 스타일 디버깅

**Sources 탭**:
- JavaScript 코드
- 브레이크포인트 설정
- 단계별 디버깅

#### 프로덕션 빌드에서 DevTools

**개발 모드** (`yarn tauri dev`):
- DevTools 사용 가능
- 모든 로그 표시

**프로덕션 빌드** (`yarn tauri build`):
- 기본적으로 DevTools 비활성화
- 필요 시 `tauri.conf.json`에서 활성화:
```json
{
  "build": {
    "devtools": true
  }
}
```

#### 디버깅 팁

**1. Svelte 반응성 디버깅**:
```javascript
$effect(() => {
  console.log('clipboardText changed:', clipboardText);
});
```

**2. 백엔드 로그와 구분**:
```javascript
console.log('[Frontend] clipboard-changed', event);
```

```rust
println!("[Backend] Clipboard changed: {}", text);
```

**3. 로그 레벨 활용**:
```javascript
console.log('정보');      // 파란색
console.warn('경고');     // 노란색
console.error('에러');    // 빨간색
console.table(data);     // 테이블 형식
```

---

## 참고 자료

### 공식 문서
- [Tauri 공식 문서](https://tauri.app/)
- [SvelteKit 공식 문서](https://kit.svelte.dev/)
- [rusqlite 문서](https://docs.rs/rusqlite/latest/rusqlite/)
- [Tokio 공식 문서](https://docs.rs/tokio/latest/tokio/)

### 관련 문서
- [`clipboard-polling-timer-guide.md`](./clipboard-polling-timer-guide.md) - 타이머 구현 상세 가이드

### 크레이트
- `rusqlite`: SQLite 데이터베이스
- `chrono`: 날짜/시간 처리
- `tokio`: 비동기 런타임
- `arboard`: 크로스 플랫폼 클립보드 (권장)
- `objc2-app-kit`: macOS Cocoa API (현재 사용 중, 스레드 제약 있음)

---

## 다음 단계

### 해결해야 할 과제

1. **클립보드 라이브러리 교체**
   - `objc2-app-kit` → `arboard`로 전환
   - 스레드 안전성 확보

2. **데이터베이스 경로 개선**
   - 현재 디렉토리 → 앱 데이터 디렉토리로 변경
   - Tauri `app.path()` API 활용

3. **폴링 시스템 완성**
   - 백그라운드 모니터링 구현
   - 이벤트 시스템 통합
   - 중복 감지 로직 추가

4. **프론트엔드 UI**
   - 클립보드 히스토리 표시
   - 실시간 업데이트 구현
   - CRUD 기능 완성

### TDD 접근

프로젝트는 TDD(Red-Green-Refactoring) 플로우를 따릅니다:

1. **Red**: 테스트 작성 (실패)
2. **Green**: 최소 구현으로 테스트 통과
3. **Refactor**: 코드 구조 개선

각 기능 구현 시 이 플로우를 따르세요.

---

**문서 작성일**: 2025-12-06
**프로젝트**: ClipboardWatcher (Tauri v2 + SvelteKit)
