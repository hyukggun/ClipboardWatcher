# thread::spawn 오류 해결 힌트

## 질문 내용

`src-tauri/src/lib.rs:55`의 `spawn_clipboard_polling_thread` 함수에서 `thread::spawn` 부분의 오류 해결 힌트 요청

## 코드 위치

파일: `src-tauri/src/lib.rs:52-73`

## 발견된 문제점

### 문제 1: `thread::spawn` 클로저 문법 오류 (55번째 줄)

```rust
thread::spawn(move |&state| {  // ❌ 잘못된 문법
```

**문제:**
- `thread::spawn`의 클로저는 **매개변수를 받지 않습니다**
- `|&state|`는 잘못된 구문입니다

**힌트:**
- `thread::spawn`의 올바른 형식: `thread::spawn(move || { ... })`
- 클로저 내부에서 사용할 값은 **외부에서 캡처**해야 합니다

**올바른 사용법:**
```rust
// 패턴: 외부 변수를 캡처하여 스레드로 이동
let value_to_move = some_value.clone();  // 또는 Arc::clone(&some_value)

thread::spawn(move || {
    // value_to_move를 여기서 사용
    // 매개변수로 받는 것이 아니라 클로저가 캡처한 것
});
```

### 문제 2: `State<AppState>` 타입의 스레드 이동 문제 (52번째 줄)

```rust
fn spawn_clipboard_polling_thread(state: State<AppState>, app_handle: AppHandle)
```

**문제:**
- `State<AppState>`는 Tauri의 특수 타입으로 **스레드 간 이동이 제한**됩니다
- `move` 클로저로 `state`를 캡처할 수 없습니다

**힌트:**
- `State<T>`는 Tauri 커맨드에서만 사용하는 타입입니다
- 스레드에서 데이터베이스를 사용하려면 다른 접근 방법이 필요합니다

**고려할 옵션:**
1. `Arc<Mutex<ClipboardDatabase>>`를 사용하여 스레드 간 공유
2. 함수 시그니처를 변경하여 데이터베이스를 직접 전달
3. 스레드 내부에서 별도의 데이터베이스 연결 생성

### 문제 3: `last_event.unwrap()` 소유권 이동 (68번째 줄)

```rust
save_clipboard_event(state, &last_event.unwrap()).unwrap();
```

**문제:**
- `unwrap()`은 `Option`에서 값을 **꺼내어(소유권 이동)** 반환합니다
- 다음 루프 반복에서 `last_event`를 사용할 수 없게 됩니다
- 이미 `last_event`는 `Some(...)`으로 설정된 직후이므로 항상 값이 있습니다

**힌트:**
- `Option::as_ref()`를 사용하여 참조만 얻기
- 또는 `unwrap()`을 사용하지 않고 직접 값에 접근하는 방법 고려
- 65번째 줄에서 방금 `Some(...)`으로 설정했으므로 바로 그 값을 사용할 수도 있습니다

### 문제 4: 함수 호출 시 인자 누락 (92번째 줄)

```rust
spawn_clipboard_polling_thread(app_handle)?;  // ❌ state 인자 누락
```

**문제:**
- 함수 정의는 `state: State<AppState>, app_handle: AppHandle`를 받는데
- 호출 시 `state`가 없습니다

## 해결 방향 힌트

### 접근 방법 1: 데이터베이스를 Arc로 감싸기

```rust
// 힌트: 이런 구조를 고려해보세요
use std::sync::Arc;

// 1. AppState 구조 유지
// 2. spawn_clipboard_polling_thread 함수 시그니처 변경
//    - State<AppState> 대신 Arc<Mutex<ClipboardDatabase>> 받기
// 3. run() 함수에서 db를 Arc로 감싸서 clone하여 전달
```

**예시 구조:**
```rust
// db를 Arc로 감싸기
let db = Arc::new(Mutex::new(ClipboardDatabase::new(db_path)?));

// 스레드로 clone 전달
let db_clone = Arc::clone(&db);
spawn_clipboard_polling_thread(db_clone, app_handle)?;

// 함수 시그니처 변경
fn spawn_clipboard_polling_thread(
    db: Arc<Mutex<ClipboardDatabase>>,
    app_handle: AppHandle
) -> Result<(), String> {
    // ...
}
```

### 접근 방법 2: 스레드 내부에서 별도 DB 연결

```rust
// 힌트: 데이터베이스 경로만 전달하고 스레드 내부에서 새 연결 생성
// 1. spawn_clipboard_polling_thread에 db_path: PathBuf 전달
// 2. 스레드 내부에서 ClipboardDatabase::new(db_path) 호출
```

**예시 구조:**
```rust
fn spawn_clipboard_polling_thread(
    db_path: PathBuf,
    app_handle: AppHandle
) -> Result<(), String> {
    thread::spawn(move || {
        // 스레드 내부에서 새 DB 연결
        let db = ClipboardDatabase::new(db_path).expect("Failed to open DB in thread");
        // ...
    });
    Ok(())
}
```

### thread::spawn의 move 키워드 이해

```rust
let app_handle = app.handle().clone();
let db_clone = Arc::clone(&db);

thread::spawn(move || {
    // move 키워드로 인해:
    // - app_handle의 소유권이 클로저로 이동
    // - db_clone의 소유권이 클로저로 이동
    //
    // 이들은 매개변수가 아니라 캡처된 변수들

    loop {
        // app_handle 사용 가능
        // db_clone 사용 가능
    }
});
```

## 수정 체크리스트

코드를 수정할 때 다음 사항들을 확인하세요:

1. ✅ `thread::spawn`의 클로저에 매개변수 제거 (`move || { ... }`)
2. ✅ 스레드에서 사용할 데이터베이스 접근 방법 결정
   - 접근 1: `Arc<Mutex<ClipboardDatabase>>` 사용
   - 접근 2: 스레드 내부에서 새 DB 연결 생성
3. ✅ `last_event.unwrap()` 대신 참조 사용
   - `as_ref()` 또는 `ref` 패턴 사용
4. ✅ 함수 호출 시 필요한 모든 인자 전달
5. ✅ `State<AppState>`를 스레드 안전한 타입으로 변환

## 핵심 인사이트

### Insight 1: Tauri State vs Rust 동시성
Tauri의 `State<T>`는 요청-응답 패턴(Tauri commands)에 특화된 타입입니다. 백그라운드 스레드에서 상태를 공유하려면 Rust의 표준 동시성 도구(`Arc`, `Mutex`)를 사용해야 합니다. 이는 Tauri의 관리 영역과 일반 Rust 코드의 경계를 명확히 하는 좋은 예입니다.

### Insight 2: move 클로저의 캡처 메커니즘
`thread::spawn(move || { ... })`의 `move` 키워드는 클로저가 참조하는 모든 외부 변수의 소유권을 클로저 안으로 이동시킵니다. 이는 스레드가 독립적으로 실행되면서도 필요한 데이터를 안전하게 소유할 수 있게 합니다. 매개변수 없이도 외부 변수를 사용할 수 있는 이유가 바로 이 캡처 메커니즘 때문입니다.

### Insight 3: Arc<Mutex<T>> 패턴
스레드 간 데이터 공유의 표준 패턴:
- `Arc`: Atomic Reference Counting - 스레드 간 소유권 공유
- `Mutex`: Mutual Exclusion - 동시 접근 제어
- 함께 사용하면 안전한 멀티스레드 데이터 접근 가능

```rust
let shared = Arc::new(Mutex::new(data));
let clone1 = Arc::clone(&shared);  // 참조 카운트 증가
let clone2 = Arc::clone(&shared);  // 참조 카운트 증가

thread::spawn(move || {
    let mut data = clone1.lock().unwrap();  // 락 획득
    // 데이터 사용
    // 스코프 끝나면 자동 unlock
});
```

## 추가 학습 포인트

### Tauri 애플리케이션의 두 가지 상태 관리 방식

1. **Tauri Commands (요청-응답)**
   - `State<T>` 사용
   - 프론트엔드에서 `invoke()` 호출
   - 동기적 처리

2. **백그라운드 작업 (이벤트 기반)**
   - `Arc<Mutex<T>>` 또는 채널 사용
   - `AppHandle::emit()` 으로 이벤트 전송
   - 비동기적 처리

현재 클립보드 폴링은 **백그라운드 작업**이므로 `Arc<Mutex<T>>` 패턴이 적합합니다.

## 참고 자료

- 파일 위치: `src-tauri/src/lib.rs:52-73, 90-94`
- 관련 개념:
  - 스레드(Thread)
  - 클로저(Closure)
  - Arc (Atomic Reference Counting)
  - Mutex (Mutual Exclusion)
  - Tauri State Management
