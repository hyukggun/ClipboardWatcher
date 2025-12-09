# 메뉴바 유틸리티 앱으로 변경하기

## 질문 내용
현재 일반 윈도우 앱을 메뉴바 유틸리티 앱 형태로 변경하고 싶음 (메뉴바 아이콘 클릭 시 메뉴바 아래에 팝업으로 표시)

## 질문에 대한 이해
macOS의 Dropbox, Raycast 같은 앱처럼 독(Dock)에 아이콘이 없고, 메뉴바에만 아이콘이 있으며 클릭 시 메뉴바 아래에 팝업 윈도우가 나타나는 형태로 변경하려는 것으로 이해했습니다.

## 구현 가이드

### 1. Tauri 설정 변경 (`src-tauri/tauri.conf.json`)

다음 사항들을 수정해야 합니다:

#### 윈도우 설정 변경
기존 `windows` 배열의 설정을 다음과 같이 수정:

```json
"windows": [
  {
    "title": "clipboardwatcher",
    "width": 400,
    "height": 600,
    "decorations": false,
    "transparent": true,
    "visible": false,
    "skipTaskbar": true,
    "alwaysOnTop": true,
    "resizable": false
  }
]
```

설정 설명:
- `decorations: false` - 타이틀바 제거
- `transparent: true` - 투명 배경 지원
- `visible: false` - 앱 시작 시 윈도우 숨김
- `skipTaskbar: true` - Dock에 아이콘 표시 안 함
- `alwaysOnTop: true` - 항상 최상위에 표시
- `resizable: false` - 크기 조절 비활성화
- `width`, `height` - 팝업에 적합한 크기로 조정

#### 시스템 트레이 설정 추가
`app` 섹션에 트레이 아이콘 설정 추가:

```json
"trayIcon": {
  "iconPath": "icons/tray-icon.png",
  "iconAsTemplate": true,
  "menuOnLeftClick": false
}
```

설정 설명:
- `iconPath` - 트레이 아이콘 경로 (static/telescope-icon.png 사용)
- `iconAsTemplate: true` - macOS에서 자동으로 다크/라이트 모드 대응
- `menuOnLeftClick: false` - 왼쪽 클릭 시 메뉴 대신 커스텀 동작 실행

### 2. Rust 백엔드 구현

#### 필요한 imports
```rust
use tauri::{
    Manager,
    PhysicalPosition,
    tray::{TrayIconBuilder, TrayIconEvent},
};
```

#### 주요 구현 사항

1. **트레이 아이콘 생성 및 이벤트 핸들러 설정**
   - `TrayIconBuilder`로 트레이 아이콘 생성
   - 클릭 이벤트 리스너 등록
   - 윈도우 표시/숨김 토글 로직 구현

2. **윈도우 위치 계산**
   - 트레이 아이콘의 화면 좌표 가져오기
   - 윈도우 너비를 고려해 중앙 정렬
   - 메뉴바 바로 아래에 배치 (보통 y = 25-30px)

   ```
   window_x = tray_icon_x - (window_width / 2) + (tray_icon_width / 2)
   window_y = menu_bar_height (약 25-30px)
   ```

3. **포커스 관리**
   - 윈도우 포커스 잃을 때 자동 숨김
   - `on_window_event` 핸들러에서 `WindowEvent::Focused(false)` 처리

4. **구현 위치**
   - `src-tauri/src/lib.rs`의 `run()` 함수 내
   - 또는 `src-tauri/src/main.rs`에서 직접 구현

#### 구현 흐름
```
1. Builder 생성
2. 트레이 아이콘 설정
3. 트레이 클릭 이벤트 핸들러 등록
4. 윈도우 이벤트 핸들러 등록 (포커스 관리)
5. 앱 실행
```

### 3. 아이콘 준비

#### 트레이 아이콘 요구사항
- **파일**: `static/telescope-icon.png` 사용
- **크기**: 16x16px 또는 32x32px (Retina용)
- **형식**: PNG (투명 배경 권장)
- **색상**: 단색 실루엣 (Template 아이콘으로 사용 시 자동 색상 조정)

#### 아이콘 위치
- 소스: `static/telescope-icon.png`
- 복사 대상: `src-tauri/icons/tray-icon.png`

### 4. 주요 구현 포인트

#### 윈도우 표시/숨김 토글
```
- 윈도우가 보이지 않을 때 → 위치 계산 후 표시
- 윈도우가 보일 때 → 숨김
```

#### 포커스 관리
```
- 윈도우 외부 클릭 → 자동 숨김
- ESC 키 → 숨김 (선택사항)
- 트레이 아이콘 재클릭 → 토글
```

#### 크로스 플랫폼 고려사항
- macOS: NSStatusBarButton 위치 기반 계산
- Windows: 시스템 트레이 위치 (우측 하단)
- Linux: 패널 위치에 따라 다름

### 5. 개발 및 테스트

#### 개발 모드 실행
```bash
yarn tauri dev
```

#### 확인 사항
- [ ] 앱 시작 시 윈도우가 보이지 않음
- [ ] Dock에 아이콘이 표시되지 않음
- [ ] 메뉴바에 트레이 아이콘이 표시됨
- [ ] 트레이 아이콘 클릭 시 윈도우가 메뉴바 아래에 나타남
- [ ] 재클릭 시 윈도우가 사라짐
- [ ] 윈도우 외부 클릭 시 자동으로 숨김

### 6. 참고 자료

- [Tauri v2 System Tray Guide](https://v2.tauri.app/reference/javascript/api/namespaces/tray/)
- [Tauri Window Management](https://v2.tauri.app/reference/javascript/api/namespaces/window/)

### 7. 추가 개선 사항 (선택)

- 트레이 메뉴 추가 (Settings, Quit 등)
- 키보드 단축키로 윈도우 토글
- 애니메이션 효과 (페이드 인/아웃)
- 윈도우 크기 조절 가능하게 변경
- 다크 모드 자동 감지 및 UI 적용
