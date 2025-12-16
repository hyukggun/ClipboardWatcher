# 클립보드 항목 삭제 후 목록 업데이트 방법

## 질문 내용
`delete_clipboard_entry` 삭제 이벤트 발생 후에 목록이 업데이트되어야 하는데 어떻게 해야 할까?

## 질문에 대한 이해

현재 코드 구조를 분석한 결과:

### 현재 구현 상태
1. **프론트엔드 (App.tsx:78-84)**
   - `handleDelete` 함수에서 프론트엔드 state를 즉시 업데이트
   - Rust 백엔드의 `delete_clipboard_entry` 커맨드 호출

2. **백엔드 (lib.rs:34-40)**
   - `delete_clipboard_entry` 커맨드가 DB에서 항목 삭제
   - 삭제 후 이벤트를 emit하지 않음

3. **비교: 추가 이벤트 (lib.rs:92-94)**
   - 클립보드 항목 추가 시 `clipboard-changed` 이벤트 emit
   - 프론트엔드가 이벤트를 listen하여 자동 업데이트

## 답변

세 가지 접근 방법을 제시합니다:

### 방법 1: 백엔드에서 삭제 이벤트 Emit (권장)
다른 창이나 세션 간 동기화가 필요한 경우 사용

**장점:**
- 여러 창이 열려있어도 모든 창에서 동기화
- 일관된 이벤트 처리 패턴 (추가와 삭제 모두 이벤트 기반)
- 프론트엔드는 이벤트만 listen하면 됨

**힌트:**
```rust
// lib.rs의 delete_clipboard_entry 수정
#[tauri::command]
fn delete_clipboard_entry(
    id: i64,
    state: State<AppState>,
    app_handle: AppHandle  // 추가
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_entry(id).map_err(|e| e.to_string())?;

    // 삭제 이벤트 emit
    app_handle.emit("clipboard-deleted", id).unwrap();
    Ok(())
}
```

```typescript
// App.tsx에 리스너 추가
listen<number>("clipboard-deleted", (event) => {
  console.log("[EVENT] clipboard-deleted received:", event.payload);
  setClipboardEvents((prev) => prev.filter((e) => e.id !== event.payload));
});
```

### 방법 2: 프론트엔드에서 직접 업데이트 (현재 구현)
단일 창 환경에서 충분한 경우

**장점:**
- 이미 구현되어 있음
- 즉각적인 UI 반응
- 백엔드 변경 불필요

**개선 포인트:**
현재 코드 (App.tsx:81)에서 `timestamp`로 필터링하는데, `id`를 사용하는 것이 더 정확합니다:
```typescript
// 현재
setClipboardEvents((prev) => prev.filter((e) => e.timestamp !== item.timestamp));

// 개선
setClipboardEvents((prev) => prev.filter((e) => e.id !== item.id));
```

### 방법 3: 삭제 후 전체 목록 재로드
간단하지만 비효율적인 방법

```typescript
const handleDelete = async (item: ClipboardEntry) => {
  await invoke("delete_clipboard_entry", { id: item.id });
  // 전체 목록 다시 로드
  const entries = await invoke<ClipboardEntryData[]>("load_clipboard_events_at_startup");
  setClipboardEvents(entries.map(e => new ClipboardEntry(e)));
};
```

## 추천 방향

1. **단일 창 앱:** 방법 2 사용 (단, id 기반 필터링으로 개선)
2. **다중 창 또는 확장 가능성:** 방법 1 사용 (이벤트 기반 동기화)

방법 1이 더 확장 가능하고 일관된 아키텍처를 제공하지만, 현재 요구사항이 단순하다면 방법 2의 개선만으로도 충분합니다.
