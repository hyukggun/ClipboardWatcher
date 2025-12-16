# 동일 ID 이벤트 2번 발생 문제 디버깅 가이드

## 질문 내용
이벤트 리스너 중복 등록 문제를 수정했지만, 여전히 동일한 ID로 2번 발생하는 것 같다.

## 질문에 대한 이해

이벤트 리스너 cleanup 문제를 해결했음에도 여전히 중복 발생한다면, 다른 원인이 있을 수 있습니다:

### 가능한 원인들

1. **React Strict Mode** - 개발 환경에서 컴포넌트를 2번 마운트
2. **백엔드에서 실제로 2번 emit** - Rust 폴링 루프가 같은 변경을 2번 감지
3. **클립보드 카운트가 2번 증가** - macOS가 복사 시 카운트를 2번 증가시킴
4. **여러 개의 폴링 스레드** - `spawn_clipboard_polling_thread`가 2번 호출됨

## 디버깅 방법

### 1. 프론트엔드 로그 확인

추가된 로그를 통해 확인할 내용:

```
[MOUNT] Registering clipboard-changed listener: abc123
[MOUNT] ✓ clipboard-changed listener registered: abc123
```

**중요:** 이 로그가 2번 나타나면 React가 컴포넌트를 2번 마운트하는 것입니다.

복사 시 로그:
```
[EVENT-abc123] clipboard-changed received: {id: 17, ...}
[EVENT-abc123] Adding entry, current count: 5
```

**만약 다른 ID가 나타나면:**
```
[EVENT-abc123] clipboard-changed received: {id: 17, ...}
[EVENT-xyz789] clipboard-changed received: {id: 17, ...}  ← 다른 리스너!
```
→ 이전 리스너가 제거되지 않은 것

### 2. Rust 백엔드 로그 확인

터미널에서 Tauri 앱을 실행할 때 나타나는 로그:

```
[POLLING] Spawning clipboard polling thread
```

**이 로그가 2번 나타나면 폴링 스레드가 중복 생성된 것입니다!**

복사 시 나타나야 할 로그:
```
[POLLING] Clipboard count changed: 0 -> 1
[POLLING] Detected text entry
[POLLING] Entry saved with id: 17
[POLLING] Emitting clipboard-changed event with id: Some(17)
```

**만약 같은 ID로 2번 나타나면:**
```
[POLLING] Clipboard count changed: 0 -> 1
[POLLING] Entry saved with id: 17
[POLLING] Emitting clipboard-changed event with id: Some(17)
[POLLING] Clipboard count changed: 1 -> 2  ← 카운트가 또 증가!
[POLLING] Entry saved with id: 18
[POLLING] Emitting clipboard-changed event with id: Some(18)
```
→ macOS가 클립보드 카운트를 2번 증가시키는 것

### 3. React Strict Mode 확인

`main.tsx` 또는 `index.tsx` 파일 확인:

```typescript
// Strict Mode가 켜져있는 경우 (개발 환경에서 2번 마운트)
<React.StrictMode>
  <App />
</React.StrictMode>

// Strict Mode 없는 경우
<App />
```

## 예상 원인별 해결 방법

### 원인 1: React Strict Mode

**확인 방법:**
- 프론트엔드 로그에서 listener ID가 2개 등장
- `[MOUNT] Registering...` 로그가 2번 나타남

**해결 방법:**
```typescript
// main.tsx
// Strict Mode를 일시적으로 제거하여 테스트
ReactDOM.createRoot(document.getElementById('root')!).render(
  // <React.StrictMode>  // 주석 처리
    <App />
  // </React.StrictMode>
);
```

**주의:** Strict Mode는 개발 환경에서만 2번 마운트하며, 프로덕션에서는 정상 작동합니다.

### 원인 2: 폴링 스레드 중복 생성

**확인 방법:**
- Rust 로그에서 `[POLLING] Spawning clipboard polling thread`가 2번 나타남

**해결 방법:**
lib.rs:128-129를 확인하여 `spawn_clipboard_polling_thread`가 1번만 호출되는지 확인:

```rust
let app_handle = app.handle().clone();
spawn_clipboard_polling_thread(app_handle.clone())?;  // 1번만 있어야 함
```

### 원인 3: 클립보드 카운트가 2번 증가

**확인 방법:**
- Rust 로그에서 `[POLLING] Clipboard count changed: 0 -> 1`와 `1 -> 2`가 연속으로 나타남
- 하나의 복사 액션에 대해 카운트가 2번 증가

**원인:**
- 일부 macOS 앱은 여러 형식으로 클립보드에 복사 (예: 텍스트 + RTF)
- 각 형식 추가 시마다 changeCount가 증가할 수 있음

**해결 방법:**
디바운싱 추가:

```rust
fn spawn_clipboard_polling_thread(app_handle: AppHandle) -> Result<(), String> {
    let mut current_count = 0;
    let mut last_content_hash: Option<u64> = None;

    thread::spawn(move || loop {
        let new_count = get_current_clipboard_count();

        if new_count == current_count {
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        current_count = new_count;

        // 내용의 해시를 계산하여 중복 체크
        let content_hash = calculate_content_hash();  // 구현 필요

        if Some(content_hash) == last_content_hash {
            println!("[POLLING] Same content detected, skipping");
            continue;
        }

        last_content_hash = Some(content_hash);

        // ... 나머지 로직
    });
}
```

## 디버깅 체크리스트

1. **앱 실행 후 터미널 로그 확인**
   - [ ] `[POLLING] Spawning...` 로그가 1번만 나타나는가?

2. **브라우저 콘솔 로그 확인**
   - [ ] `[MOUNT] Registering...` 로그가 몇 번 나타나는가?
   - [ ] listener ID가 몇 개인가?

3. **복사 액션 후 로그 확인**
   - [ ] Rust: `[POLLING] Clipboard count changed` 로그가 몇 번?
   - [ ] Rust: `[POLLING] Emitting...` 로그가 몇 번?
   - [ ] Frontend: `[EVENT-xxx] clipboard-changed received` 로그가 몇 번?

4. **결과 분석**
   - Rust가 1번 emit하는데 프론트엔드가 2번 받으면 → 리스너 중복
   - Rust가 2번 emit하면 → 백엔드 문제 (카운트 또는 스레드 중복)

## 다음 단계

위 로그를 확인한 후:
1. 로그 전체를 공유하면 정확한 원인 파악 가능
2. 원인에 따라 적절한 해결책 적용
