# Tauri 앱 배포 방법

## 질문 내용
Tauri 앱을 배포하는 방법에 대해 알려줘.

## 질문에 대한 이해
ClipboardWatcher Tauri v2 애플리케이션을 프로덕션 환경에 배포하기 위한 빌드 및 배포 프로세스에 대한 가이드를 요청하신 것으로 이해합니다.

## 답변

### 1. 빌드 준비

#### 1.1 설정 확인
현재 프로젝트의 `src-tauri/tauri.conf.json` 설정을 확인하세요:

- **productName**: "clipboardwatcher" - 애플리케이션 이름
- **version**: "0.1.0" - 버전 정보 (릴리스 전 업데이트 필요)
- **identifier**: "com.hyukmac.clipboardwatcher" - 고유 식별자
- **bundle.targets**: "all" - 현재 플랫폼의 모든 번들 타입 생성

#### 1.2 버전 관리
배포 전 버전 업데이트가 필요한 파일:
- `src-tauri/tauri.conf.json` - version 필드
- `src-tauri/Cargo.toml` - version 필드
- `package.json` - version 필드

### 2. 프로덕션 빌드

#### 2.1 기본 빌드 명령어
```bash
yarn tauri build
```

이 명령어는 다음을 수행합니다:
1. `yarn build` 실행 (프론트엔드 빌드)
2. Rust 백엔드 릴리스 빌드
3. 플랫폼별 설치 파일 생성

#### 2.2 빌드 출력물 위치
빌드된 파일은 `src-tauri/target/release/bundle/` 디렉토리에 생성됩니다:

**macOS** (현재 개발 환경):
- `dmg/` - DMG 이미지 파일
- `macos/` - .app 번들
- 위치: `src-tauri/target/release/bundle/macos/clipboardwatcher.app`

**Windows** (크로스 빌드 불가):
- `msi/` - Windows Installer
- `nsis/` - NSIS 설치 프로그램

**Linux**:
- `deb/` - Debian 패키지
- `appimage/` - AppImage 번들
- `rpm/` - RPM 패키지

### 3. 플랫폼별 배포 전략

#### 3.1 macOS 배포

**Option 1: DMG 파일 직접 배포**
```bash
yarn tauri build
# 생성된 DMG 파일 배포: src-tauri/target/release/bundle/dmg/clipboardwatcher_0.1.0_*.dmg
```

**Option 2: 코드 서명 (추천)**
```bash
# Apple Developer 계정 필요
# tauri.conf.json에 서명 설정 추가 필요
```

힌트:
- 코드 서명 없이 배포 시 사용자는 "신뢰할 수 없는 개발자" 경고를 받게 됩니다
- Gatekeeper 우회를 위해 사용자가 "시스템 설정 > 개인정보 보호 및 보안"에서 수동 허용 필요

#### 3.2 Windows 배포

Windows에서 빌드 필요 (macOS에서 크로스 빌드 불가):
```bash
yarn tauri build
# MSI 또는 NSIS 설치 프로그램 생성
```

#### 3.3 Linux 배포

Linux 환경에서 빌드:
```bash
yarn tauri build
# AppImage, DEB, RPM 등 생성
```

### 4. 배포 최적화

#### 4.1 번들 타겟 선택
특정 포맷만 빌드하려면 `tauri.conf.json` 수정:

```json
"bundle": {
  "active": true,
  "targets": ["dmg"],  // macOS에서 DMG만 생성
  "icon": [...]
}
```

가능한 타겟:
- macOS: "dmg", "app"
- Windows: "msi", "nsis"
- Linux: "deb", "appimage", "rpm"

#### 4.2 빌드 최적화
릴리스 빌드 최적화를 위해 `src-tauri/Cargo.toml`에 다음 추가 고려:

```toml
[profile.release]
opt-level = "z"     # 크기 최적화
lto = true          # Link Time Optimization
codegen-units = 1   # 단일 코드 생성 유닛
strip = true        # 심볼 제거
```

### 5. 배포 채널

#### 5.1 직접 배포
- GitHub Releases 활용
- 웹사이트에서 직접 다운로드 제공
- 자체 호스팅 서버 사용

#### 5.2 앱 스토어 배포
- **macOS**: Mac App Store (별도 프로비저닝 필요)
- **Windows**: Microsoft Store
- **Linux**: Snapcraft, Flathub

#### 5.3 자동 업데이트 (향후 고려)
Tauri의 업데이트 플러그인 활용:
```bash
# src-tauri/Cargo.toml에 추가
tauri-plugin-updater = "2"
```

### 6. 배포 전 체크리스트

- [ ] 모든 기능 테스트 완료
- [ ] 버전 번호 업데이트 (3곳: tauri.conf.json, Cargo.toml, package.json)
- [ ] 아이콘 파일 확인 (icons/ 디렉토리)
- [ ] 프로덕션 빌드 테스트 (`yarn tauri build`)
- [ ] 생성된 번들 파일 실행 테스트
- [ ] 릴리스 노트 작성
- [ ] (선택) 코드 서명 설정
- [ ] (선택) 자동 업데이트 구성

### 7. 빠른 시작

현재 macOS 환경에서 배포용 DMG 생성:

```bash
# 1. 프론트엔드 타입 체크
yarn check

# 2. 프로덕션 빌드
yarn tauri build

# 3. 빌드 결과 확인
ls -lh src-tauri/target/release/bundle/dmg/

# 4. 생성된 DMG 테스트
open src-tauri/target/release/bundle/dmg/clipboardwatcher_0.1.0_*.dmg
```

### 추가 참고사항

- **디버그 빌드 vs 릴리스 빌드**: `yarn tauri dev`는 디버그 빌드, `yarn tauri build`는 최적화된 릴리스 빌드입니다.
- **크로스 플랫폼 빌드**: 각 플랫폼별 빌드는 해당 OS에서 수행해야 합니다. CI/CD (GitHub Actions 등)를 통해 자동화 가능합니다.
- **데이터베이스 파일**: 현재 `clipboard_history.db`는 `.gitignore`에 추가되어 있으므로, 앱 초기 실행 시 데이터베이스 생성 로직이 필요합니다.
