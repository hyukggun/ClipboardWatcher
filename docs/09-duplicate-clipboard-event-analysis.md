# 클립보드 복사 시 이벤트 2번 발생 문제 분석

## 질문 내용
수정을 해서 정상적으로 반영되고 있지만, 한번 복사 시에 이벤트가 2번 발생하는 이유를 분석해달라.

## 문제 분석

### 발견된 주요 원인: 이벤트 리스너 중복 등록

**App.tsx:51-67의 문제점:**

```typescript
let unlisten: any;

// 첫 번째 리스너
listen<ClipboardEntryData>("clipboard-changed", (event) => {
  // ...
}).then((fn) => {
  unlisten = fn;  // unlisten에 첫 번째 cleanup 함수 저장
});

// 두 번째 리스너
listen<number>("clipboard-deleted", (event) => {
  // ...
}).then((fn) => {
  unlisten = fn;  // 같은 unlisten 변수를 덮어씀! ❌
});

return () => {
  if (unlisten) unlisten();  // 마지막 리스너만 cleanup됨
};
```

**문제의 결과:**
1. `clipboard-changed` 리스너의 cleanup 함수가 `clipboard-deleted`에 의해 덮어쓰여짐
2. `useEffect` cleanup 시 `clipboard-changed` 리스너가 제거되지 않음
3. 컴포넌트가 재마운트되면 이전 리스너가 남아있어 중복 등록됨

### 추가 원인 가능성

#### 1. React Strict Mode
React 18의 Strict Mode는 개발 환경에서 컴포넌트를 2번 마운트합니다:
```
Mount → Unmount → Remount (개발 환경)
```

이벤트 리스너 cleanup이 제대로 안 되면 이전 리스너가 남아서 중복 실행됩니다.

#### 2. 클립보드 카운트 변화
lib.rs:67-72를 보면 클립보드 카운트를 기반으로 변경 감지:
```rust
let new_count = get_current_clipboard_count();
println!("New clipboard count: {:?}", new_count);
if new_count == current_count {
    thread::sleep(Duration::from_secs(1));
    continue;
}
```

**의문점:** `get_current_clipboard_count()` 함수가 복사 1번에 카운트를 2번 증가시키는가?

## 해결 방법

### 방법 1: 각 리스너의 cleanup 함수를 개별적으로 저장 (권장)

```typescript
useEffect(() => {
  console.log("[INIT] React App mounting");

  if (typeof (window as any).__TAURI_INTERNALS__ === "undefined") {
    console.error("[ERROR] ⚠️ NOT RUNNING IN TAURI CONTEXT ⚠️");
    return;
  }

  // 각 리스너의 cleanup 함수를 별도로 저장
  let unlistenClipboardChanged: any;
  let unlistenClipboardDeleted: any;

  // clipboard-changed 리스너
  listen<ClipboardEntryData>("clipboard-changed", (event) => {
    console.log("[EVENT] clipboard-changed received:", event.payload);
    const entry = new ClipboardEntry(event.payload);
    setClipboardEvents((prev) => [entry, ...prev]);
  }).then((fn) => {
    unlistenClipboardChanged = fn;
  });

  // clipboard-deleted 리스너
  listen<number>("clipboard-deleted", (event) => {
    console.log("[EVENT] clipboard-deleted received:", event.payload);
    setClipboardEvents((prev) => prev.filter((e) => e.id !== event.payload));
  }).then((fn) => {
    unlistenClipboardDeleted = fn;
  });

  // 초기 데이터 로드
  invoke<ClipboardEntryData[]>("load_clipboard_events_at_startup")
    .then((entries) => {
      const clipboardEntries = entries.map((e) => new ClipboardEntry(e));
      setClipboardEvents(clipboardEntries);
    });

  // cleanup: 모든 리스너 제거
  return () => {
    if (unlistenClipboardChanged) unlistenClipboardChanged();
    if (unlistenClipboardDeleted) unlistenClipboardDeleted();
  };
}, []);
```

### 방법 2: async/await로 순차 처리

```typescript
useEffect(() => {
  let unlisteners: Array<() => void> = [];

  const setup = async () => {
    // 리스너들을 순차적으로 등록
    const unlisten1 = await listen<ClipboardEntryData>("clipboard-changed", (event) => {
      const entry = new ClipboardEntry(event.payload);
      setClipboardEvents((prev) => [entry, ...prev]);
    });

    const unlisten2 = await listen<number>("clipboard-deleted", (event) => {
      setClipboardEvents((prev) => prev.filter((e) => e.id !== event.payload));
    });

    unlisteners = [unlisten1, unlisten2];

    // 초기 데이터 로드
    const entries = await invoke<ClipboardEntryData[]>("load_clipboard_events_at_startup");
    setClipboardEvents(entries.map(e => new ClipboardEntry(e)));
  };

  setup();

  return () => {
    unlisteners.forEach(unlisten => unlisten());
  };
}, []);
```

## 추가 디버깅 방법

### 1. 이벤트 리스너 카운트 확인
```typescript
let listenerCount = 0;

listen<ClipboardEntryData>("clipboard-changed", (event) => {
  listenerCount++;
  console.log(`[EVENT] clipboard-changed #${listenerCount}:`, event.payload);
  // ...
});
```

### 2. Rust 로그 확인
lib.rs:68에 이미 로그가 있으므로 터미널에서 확인:
```
New clipboard count: 1
New clipboard count: 2  // 1번 복사에 2번 나온다면 백엔드 문제
```

### 3. base 모듈의 get_current_clipboard_count 확인
`base.rs` 파일을 확인하여 카운트가 왜 2번 증가하는지 확인 필요

## 권장 조치

1. **즉시 수정:** 방법 1을 적용하여 이벤트 리스너 cleanup 문제 해결
2. **디버깅:** Rust 로그를 확인하여 클립보드 카운트가 실제로 2번 증가하는지 확인
3. **추가 조사:** 필요시 `base.rs`의 `get_current_clipboard_count` 함수 검토

## 예상 결과

방법 1 적용 후에도 문제가 지속된다면:
- Rust의 `get_current_clipboard_count()`가 1번 복사에 2번 증가하는 것이 원인
- `base.rs` 파일의 클립보드 모니터링 로직을 확인해야 함
