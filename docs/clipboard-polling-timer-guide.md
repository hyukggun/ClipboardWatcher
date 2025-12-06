# 클립보드 폴링 타이머 구현 가이드

## 질문 내용

> 현재는 `@src-tauri/src/base.rs`에서 `get_clipboard_text`를 호출해야만 복사한 텍스트를 가져올 수 있어. 이 구조에서 벗어나 폴링 방식 + 타이머를 통해 일정 간격으로 계속해서 텍스트를 가져오도록 하려고 해. 타이머를 사용하는 방법을 알려줘.

---

## 질문에 대한 이해

### 현재 구조의 문제점
- **수동 호출 방식**: 프론트엔드에서 `get_clipboard_text()` Tauri 커맨드를 명시적으로 호출해야만 클립보드 내용을 가져올 수 있음
- **실시간성 부족**: 사용자가 클립보드에 복사한 내용을 즉시 감지하지 못함

### 원하는 구조
- **자동 폴링**: 백그라운드에서 일정 간격으로 클립보드를 확인
- **변경 감지**: 클립보드 내용이 변경되었을 때 자동으로 데이터베이스에 저장
- **비동기 처리**: 메인 스레드를 블로킹하지 않고 백그라운드에서 실행

---

## 답변: Rust에서 타이머를 사용한 폴링 구현 방법

### 1. 접근 방법 개요

Rust에서 주기적인 작업을 수행하는 방법은 크게 3가지가 있습니다:

#### Option A: `std::thread` + `std::thread::sleep`
```
장점: 표준 라이브러리만 사용, 간단한 구현
단점: 타이머 취소가 어려움, 유연성 부족
적합한 경우: 간단한 폴링, 앱 종료 시까지 계속 실행
```

#### Option B: `tokio` + `tokio::time::interval`
```
장점: 비동기 처리, 타이머 관리 용이, Tauri와 잘 통합됨
단점: tokio 런타임 필요, 복잡도 증가
적합한 경우: 복잡한 비동기 작업, 여러 타이머 관리
```

#### Option C: Tauri의 `AppHandle` + Custom Event System
```
장점: Tauri 생태계와 완벽한 통합, 프론트엔드와 양방향 통신
단점: Tauri 특화된 코드
적합한 경우: 프론트엔드에서 실시간 업데이트가 필요한 경우
```

---

### 2. 구현 전략

#### Step 1: 의존성 추가
`src-tauri/Cargo.toml`에 필요한 크레이트를 추가해야 합니다:
- `tokio` (이미 Tauri가 사용 중일 가능성 높음)
- 또는 표준 라이브러리의 `std::thread` 사용

#### Step 2: 타이머 설정
폴링 간격을 결정해야 합니다:
- **너무 짧은 간격 (< 100ms)**: CPU 사용량 증가, 배터리 소모
- **적절한 간격 (500ms - 2s)**: 반응성과 효율성의 균형
- **긴 간격 (> 5s)**: 효율적이지만 변경 감지 지연

#### Step 3: 중복 감지 로직
같은 내용을 여러 번 저장하지 않도록 해야 합니다:
- 이전 클립보드 내용을 변수에 저장
- 새로운 내용과 비교하여 변경된 경우만 처리
- 빈 문자열 처리에 주의

#### Step 4: 앱 상태 통합
`lib.rs`의 `AppState`와 통합이 필요합니다:
- 타이머가 `AppState`의 데이터베이스에 접근할 수 있어야 함
- `Arc<Mutex<T>>` 또는 Tauri의 `State`를 통한 공유

---

### 3. 구현 위치 및 구조

#### 새 모듈 생성 권장: `src-tauri/src/clipboard_monitor.rs`

이 모듈에 다음 기능을 구현하세요:
- `start_clipboard_monitoring()` - 폴링 시작
- `stop_clipboard_monitoring()` - 폴링 중지 (선택적)
- 내부 상태 관리 (이전 클립보드 내용)

#### `lib.rs`에서 통합
`run()` 함수에서 앱이 시작될 때 모니터링을 시작합니다:
- Tauri Builder 설정 후
- 앱 핸들을 클로저에 전달
- 백그라운드 스레드 또는 비동기 태스크 시작

---

### 4. 주의사항 및 트레이드오프

#### 성능 고려사항
- **CPU 사용**: 폴링 간격이 짧을수록 CPU 사용량 증가
- **메모리**: 클립보드에 큰 텍스트가 복사되면 메모리 사용 증가
- **배터리**: 모바일이나 노트북에서 배터리 소모 고려

#### 스레드 안전성
- `base.rs`의 `get_clipboard_text()`는 macOS의 `NSPasteboard`를 사용
- macOS의 Pasteboard API는 메인 스레드에서 호출해야 할 수도 있음
- 크로스 플랫폼 지원 시 각 플랫폼의 제약사항 확인 필요

#### 에러 처리
- 클립보드 접근 실패 (권한 문제)
- 데이터베이스 저장 실패
- 타이머 중지 시 cleanup

#### 대안: 이벤트 기반 접근
폴링 대신 클립보드 변경 이벤트를 직접 감지하는 방법도 있습니다:
- macOS: `NSPasteboardDidChangeNotification`
- 더 효율적이지만 구현 복잡도 증가
- 플랫폼별 구현 필요

---

### 5. 테스트 전략

#### TDD 접근 (프로젝트 요구사항)
1. **Red**: 타이머가 주기적으로 호출되는지 테스트 작성
2. **Green**: 최소 구현으로 테스트 통과
3. **Refactor**: 코드 구조 개선

#### 테스트 케이스 예시
- 타이머가 정해진 간격으로 실행되는가?
- 중복 내용이 저장되지 않는가?
- 빈 클립보드 처리가 올바른가?
- 앱 종료 시 타이머가 정리되는가?

---

### 6. 구현 힌트

#### Tokio를 사용한 패턴 예시 (의사코드)
```rust
// 1. 비동기 함수로 모니터링 로직 구현
async fn monitor_clipboard(app_handle: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut last_content = String::new();

    loop {
        interval.tick().await;

        // 클립보드 내용 가져오기
        // 이전 내용과 비교
        // 변경되었으면 데이터베이스에 저장
        // 에러 처리
    }
}

// 2. lib.rs의 run() 함수에서 시작
tauri::Builder::default()
    .setup(|app| {
        let app_handle = app.handle().clone();
        tokio::spawn(async move {
            monitor_clipboard(app_handle).await;
        });
        Ok(())
    })
```

#### 표준 스레드를 사용한 패턴 예시 (의사코드)
```rust
// 1. 스레드로 모니터링 로직 구현
fn monitor_clipboard(/* 필요한 상태 전달 */) {
    let mut last_content = String::new();

    loop {
        std::thread::sleep(Duration::from_secs(1));

        // 클립보드 내용 가져오기
        // 이전 내용과 비교
        // 변경되었으면 데이터베이스에 저장
        // 에러 처리
    }
}

// 2. lib.rs의 run() 함수에서 시작
std::thread::spawn(|| {
    monitor_clipboard(/* 상태 전달 */);
});
```

---

### 7. 다음 단계

구현을 시작하기 전에 결정해야 할 사항:

1. **폴링 간격**: 몇 초마다 확인할 것인가? (권장: 1-2초)
2. **사용할 방법**: tokio vs std::thread?
3. **중지 기능**: 사용자가 모니터링을 중지할 수 있어야 하는가?
4. **프론트엔드 통신**: 새로운 클립보드 항목이 추가되면 프론트엔드에 알림을 보낼 것인가?

이러한 결정을 내린 후 TDD 플로우에 따라 구현을 시작하세요!

---

## 참고 자료

- Tokio 공식 문서: https://docs.rs/tokio/latest/tokio/time/index.html
- Tauri Setup Hook: https://tauri.app/develop/plugins/#setup-hook
- Rust 스레드 가이드: https://doc.rust-lang.org/book/ch16-01-threads.html
