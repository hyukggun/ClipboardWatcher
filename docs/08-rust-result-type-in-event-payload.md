# Rust Result 타입 이벤트 페이로드 처리

## 질문 내용
타입스크립트 로그에서 `[EVENT] clipboard-deleted received: {Ok: 17}` 과 같이 출력되는데 어떻게 값을 뽑아내는가?

## 질문에 대한 이해

### 문제의 원인
현재 Rust 코드 (lib.rs:37-39)를 보면:

```rust
let id = db.delete_entry(id).map_err(|e| e.to_string());
// id는 Result<i64, String> 타입
app_handle.emit("clipboard-deleted", id).map_err(|e| e.to_string())?;
// Result를 그대로 emit
```

- `db.delete_entry(id)`는 `Result<i64, String>`을 반환
- `map_err`는 에러 타입만 변환하고 여전히 `Result`
- `Result` 타입을 그대로 emit하면 JSON으로 `{Ok: value}` 또는 `{Err: error}` 형태로 직렬화됨

### Rust Result 타입의 JSON 직렬화
```rust
Result::Ok(17)   → {Ok: 17}
Result::Err("error") → {Err: "error"}
```

## 답변

두 가지 해결 방법이 있습니다:

### 방법 1: Rust에서 Result 처리 후 값만 emit (권장)

**백엔드 수정 힌트:**
```rust
#[tauri::command]
fn delete_clipboard_entry(
    id: i64,
    state: State<AppState>,
    app_handle: AppHandle
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Result를 풀어서 값만 추출
    let deleted_id = db.delete_entry(id).map_err(|e| e.to_string())?;

    // 값만 emit
    app_handle.emit("clipboard-deleted", deleted_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

**프론트엔드에서 사용:**
```typescript
listen<number>("clipboard-deleted", (event) => {
  console.log("[EVENT] clipboard-deleted received:", event.payload);
  // event.payload는 이제 17 (숫자)
  setClipboardEvents((prev) => prev.filter((e) => e.id !== event.payload));
});
```

### 방법 2: 프론트엔드에서 Result 타입 파싱

현재 백엔드 코드를 수정하지 않고 프론트엔드에서 처리:

```typescript
type RustResult<T, E> =
  | { Ok: T }
  | { Err: E };

listen<RustResult<number, string>>("clipboard-deleted", (event) => {
  const payload = event.payload;

  if ('Ok' in payload) {
    // 성공한 경우
    const id = payload.Ok;
    console.log("[EVENT] Deleted ID:", id);
    setClipboardEvents((prev) => prev.filter((e) => e.id !== id));
  } else if ('Err' in payload) {
    // 에러가 발생한 경우
    console.error("[ERROR] Delete failed:", payload.Err);
  }
});
```

## 추천 방향

**방법 1을 권장**합니다:
- Rust에서 에러 처리를 완료하고 성공한 값만 emit
- 프론트엔드 코드가 더 간단해짐
- 타입 안정성 향상
- 에러는 Rust 함수의 `Result` 반환값으로 처리 (invoke 호출 시 catch)

### 전체 흐름
```
Rust delete_entry → Result<i64, String>
       ↓
    ? operator → 실패하면 함수가 Err 반환
       ↓
   성공 시 i64 값만 추출
       ↓
    emit(id) → 프론트엔드로 숫자만 전달
```

## 추가 참고사항

현재 코드의 또 다른 문제점 (lib.rs:37):
```rust
let id = db.delete_entry(id).map_err(|e| e.to_string());
```

변수명이 겹칩니다. 더 명확하게:
```rust
let deleted_id = db.delete_entry(id).map_err(|e| e.to_string())?;
```
