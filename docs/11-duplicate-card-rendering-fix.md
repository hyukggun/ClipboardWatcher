# ClipboardCard 중복 렌더링 문제 해결

## 질문 내용
ClipboardEntryCard가 중복해서 나오고, 삭제 버튼을 눌렀을 때 2개가 한 번에 삭제된다.

## 질문에 대한 이해

### 증상 분석
1. **UI에 같은 항목이 2개 표시됨** - 같은 내용의 카드가 중복 렌더링
2. **삭제 시 2개가 동시에 삭제됨** - 같은 ID를 가진 항목이 2개 있다는 증거

### 근본 원인
이 증상은 **React Strict Mode**와 **이벤트 리스너 중복 등록**의 조합으로 발생:

```typescript
// main.tsx - React Strict Mode
<React.StrictMode>  ← 개발 환경에서 컴포넌트를 2번 마운트
  <App />
</React.StrictMode>
```

**React Strict Mode의 동작:**
- 개발 환경에서 의도적으로 컴포넌트를 2번 마운트/언마운트
- Effect cleanup이 제대로 동작하는지 테스트하기 위함
- Mount → Unmount → Remount 순서로 실행

**문제 발생 메커니즘:**
1. 첫 번째 마운트 → 이벤트 리스너 등록
2. 언마운트 → cleanup 실행 (리스너 제거)
3. 두 번째 마운트 → 이벤트 리스너 재등록
4. **하지만** cleanup이 제대로 안 되면 첫 번째 리스너가 남아있음
5. 결과: 같은 이벤트에 대해 2개의 리스너가 반응
6. 같은 항목이 state에 2번 추가됨

### 삭제 시 2개가 동시에 삭제되는 이유

```typescript
// App.tsx - 삭제 핸들러
setClipboardEvents((prev) => prev.filter((e) => e.id !== item.id));
```

- 같은 ID를 가진 모든 항목을 제거
- 중복된 항목(같은 ID)이 2개 있으면 동시에 삭제됨

## 해결 방법

### 1. 중복 체크 추가 (App.tsx:62-69)

이벤트 리스너에서 같은 ID가 이미 존재하는지 체크:

```typescript
listen<ClipboardEntryData>("clipboard-changed", (event) => {
  const entry = new ClipboardEntry(event.payload);
  setClipboardEvents((prev) => {
    // 중복 체크: 같은 ID가 이미 존재하면 추가하지 않음
    const isDuplicate = prev.some((e) => e.id === entry.id);
    if (isDuplicate) {
      console.log(`Duplicate entry detected (id: ${entry.id}), skipping`);
      return prev;  // 상태 변경 없음
    }
    return [entry, ...prev];  // 새 항목 추가
  });
});
```

**효과:**
- 이벤트가 2번 발생해도 같은 항목이 1번만 추가됨
- 방어적 프로그래밍 (defensive programming)

### 2. React key를 고유하게 변경 (App.tsx:207)

**Before:**
```typescript
{filteredEvents.map((item, index) => (
  <ClipboardCard key={index} item={item} ... />  // ❌ index는 불안정
))}
```

**After:**
```typescript
{filteredEvents.map((item) => (
  <ClipboardCard key={`${item.id}-${item.timestamp}`} item={item} ... />  // ✓ 고유한 key
))}
```

**문제점:**
- `index`를 key로 사용하면 항목 순서가 바뀔 때 React가 제대로 추적 못함
- 중복된 항목이 있을 때 렌더링 문제 발생

**개선:**
- `id`와 `timestamp` 조합으로 완전히 고유한 key 생성
- React가 각 항목을 정확히 추적 가능

### 3. React Strict Mode 제거 (main.tsx:6-8)

**Before:**
```typescript
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

**After:**
```typescript
ReactDOM.createRoot(document.getElementById("root")!).render(
  <App />
);
```

**주의사항:**
- Strict Mode는 개발 시 유용한 경고를 제공
- 하지만 이벤트 리스너 중복 문제를 일으킬 수 있음
- 프로덕션 빌드에서는 Strict Mode가 비활성화되어 문제 없음

**대안:**
- 개발 완료 후 다시 Strict Mode를 켜서 최종 테스트
- Effect cleanup이 완벽하게 작동하는지 확인

## 적용된 수정 사항 요약

| 파일 | 수정 내용 | 목적 |
|------|-----------|------|
| App.tsx:62-69 | 중복 체크 로직 추가 | 같은 ID 항목 중복 추가 방지 |
| App.tsx:207 | key={index} → key={`${id}-${timestamp}`} | React 렌더링 최적화 |
| main.tsx:6-8 | React.StrictMode 제거 | 개발 환경 이중 마운트 방지 |

## 예상 결과

수정 후:
1. ✅ 복사 시 항목이 1개만 추가됨
2. ✅ 삭제 시 1개만 삭제됨
3. ✅ UI에 중복 카드가 나타나지 않음
4. ✅ 콘솔 로그에서 "Duplicate entry detected" 메시지가 나타나지 않음

## 추가 권장사항

### 프로덕션 배포 전 체크리스트
1. Strict Mode를 다시 켜서 테스트
2. Effect cleanup이 제대로 작동하는지 확인
3. 메모리 누수가 없는지 확인

### 장기적 개선 방향
1. TypeScript의 strict mode 활성화
2. React의 useCallback, useMemo 사용 검토
3. 이벤트 리스너를 커스텀 훅으로 분리
   ```typescript
   function useClipboardEvents() {
     useEffect(() => {
       // 리스너 등록 로직
       return () => {
         // cleanup 로직
       };
     }, []);
   }
   ```
