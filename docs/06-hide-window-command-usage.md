# Hide Window Command 사용 가이드

## 구현 내용

### 백엔드 (Rust)

`src-tauri/src/lib.rs`에 `hide_window` Tauri command가 추가되었습니다:

```rust
#[tauri::command]
fn hide_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}
```

**주요 특징:**
- `app_handle.get_webview_window("main")`으로 메인 윈도우 참조를 가져옵니다
- `window.hide()`를 호출하여 윈도우를 숨깁니다
- 윈도우를 찾지 못한 경우 에러를 반환합니다
- 모든 에러는 `String` 타입으로 변환되어 프론트엔드로 전달됩니다

### 프론트엔드 (TypeScript/Svelte)

프론트엔드에서 이 command를 호출하는 방법:

```typescript
import { invoke } from '@tauri-apps/api/core';

// 기본 사용법
async function hideWindow() {
  try {
    await invoke('hide_window');
    console.log('Window hidden successfully');
  } catch (error) {
    console.error('Failed to hide window:', error);
  }
}

// Svelte 컴포넌트에서 사용
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  async function handleHideWindow() {
    try {
      await invoke('hide_window');
    } catch (error) {
      console.error('Failed to hide window:', error);
    }
  }
</script>

<button on:click={handleHideWindow}>
  Hide Window
</button>
```

## 사용 예시

### 1. ESC 키로 윈도우 숨기기

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  onMount(() => {
    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        invoke('hide_window').catch(console.error);
      }
    };

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>
```

### 2. 버튼 클릭으로 윈도우 숨기기

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  const hideWindow = () => {
    invoke('hide_window').catch(console.error);
  };
</script>

<button on:click={hideWindow}>Close</button>
```

### 3. 특정 작업 완료 후 자동으로 숨기기

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  async function copyToClipboard(text: string) {
    // 클립보드에 복사
    await navigator.clipboard.writeText(text);

    // 윈도우 숨기기
    await invoke('hide_window');
  }
</script>
```

## 관련 Tauri API

현재 구현된 윈도우 관리 기능:
- `hide_window`: 윈도우를 숨김 (새로 추가됨)
- Tray icon 클릭: 윈도우 show/hide 토글 (기존 기능)
- Focus 상실: 자동으로 윈도우 숨김 (기존 기능, `on_window_event`에서 처리)

## 참고사항

- 윈도우가 숨겨져도 애플리케이션은 백그라운드에서 계속 실행됩니다
- Tray icon을 클릭하거나 "Open" 메뉴를 선택하면 다시 윈도우를 표시할 수 있습니다
- 현재 코드는 포커스를 잃으면 자동으로 윈도우가 숨겨지도록 설정되어 있습니다 (`on_window_event` 참조)
