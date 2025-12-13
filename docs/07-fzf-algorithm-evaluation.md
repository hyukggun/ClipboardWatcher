# FZF Algorithm Implementation Evaluation

## 개요

`calculate_fzf_score` 함수는 fuzzy finding 알고리즘을 구현하여 query 문자열이 target 문자열과 얼마나 잘 매칭되는지 점수를 계산합니다.

**위치**: `src-tauri/src/fzf.rs`

## FZF 알고리즘이란?

FZF (Fuzzy Finder)는 부분 문자열 매칭 알고리즘으로, 다음과 같은 특징을 가집니다:

1. **Subsequence Matching**: Query의 모든 문자가 순서대로 text에 존재하면 매칭
   - 예: "hw" matches "hello_world" (h at 0, w at 6)

2. **Scoring System**: 매칭 품질에 따라 점수 부여
   - 연속된 매칭: 높은 점수
   - Gap이 있는 매칭: 낮은 점수
   - 특정 위치 (단어 시작, CamelCase): 보너스 점수

3. **Dynamic Programming**: 효율적인 점수 계산을 위해 DP 사용

## 구현 분석

### 상수 정의

```rust
const INITIAL_SCORE: i32 = 5;      // 문자열 시작 위치 보너스
const BOUNDARY_SCORE: i32 = 3;     // 단어 경계 보너스 (gap penalty보다 큼)
const CAMEL_CASE_SCORE: i32 = 2;   // CamelCase 경계 보너스
const MATCH_SCORE: i32 = 10;       // 기본 매칭 점수
const GAP_SCORE: i32 = -2;         // Gap penalty
const NO_SCORE: i32 = -10000;      // 매칭 없음
```

### 핵심 알고리즘: Dynamic Programming

#### 1. Bonus Score 계산 (`calculate_bonus_score`)

각 문자 위치에 대한 보너스 점수를 미리 계산:

```rust
fn calculate_bonus_score(text: &String) -> Vec<i32>
```

**보너스가 주어지는 경우:**
- **첫 문자** (index 0): `INITIAL_SCORE` (5점)
- **단어 경계** 다음: `BOUNDARY_SCORE` (3점)
  - 구분자: `/`, `_`, `-`, `.`, ` ` (공백)
- **CamelCase**: `CAMEL_CASE_SCORE` (2점)
  - 소문자 다음 대문자

**예시:**
```
text:  "hello_world"
bonus: [5, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0]
        ^              ^
        첫문자         단어경계

text:  "helloWorld"
bonus: [5, 0, 0, 0, 0, 2, 0, 0, 0, 0]
        ^              ^
        첫문자         CamelCase
```

#### 2. FZF Score 계산 (`calculate_fzf_score`)

```rust
pub fn calculate_fzf_score(text: &String, query: &String) -> Vec<i32>
```

**알고리즘 구조:**

2D DP 테이블을 1D로 최적화하여 구현:
- `prev_score`: 이전 query 문자의 점수 (이전 행)
- `current_score`: 현재 query 문자의 점수 (현재 행)

**핵심 로직:**

```rust
for (i, q_char) in query.chars().enumerate() {
    let mut current_score = vec![NO_SCORE; text.len()];
    let mut current_best_score = NO_SCORE;

    for (j, t_char) in text.chars().enumerate() {
        // 1. Query의 첫 문자 (i == 0)
        if i == 0 {
            current_best_score = 0;  // Gap penalty 없음
        }
        // 2. Query의 두 번째 문자부터 (i > 0)
        else {
            // 2-1. 현재 위치까지의 best score에 gap penalty
            if current_best_score > NO_SCORE {
                current_best_score += GAP_SCORE;
            }

            // 2-2. 이전 query 문자의 직전 위치 점수 고려
            // prev_score[j-1]: 이전 query 문자가 text[j-1]에서 매칭된 점수
            // 이를 기반으로 현재 query 문자가 text[j]에서 매칭
            if j > 0 {
                let score_from_prev_row = prev_score[j - 1];
                if score_from_prev_row > current_best_score {
                    current_best_score = score_from_prev_row;
                }
            }
        }

        // 3. 문자 매칭 시 점수 계산
        if q_char.eq_ignore_ascii_case(&t_char) {
            current_score[j] = current_best_score + bonus_score[j] + MATCH_SCORE;
        }
    }

    prev_score = current_score;
}
```

### 알고리즘 단계별 예시

**Query**: "hw"
**Text**: "hello_world"

#### 초기화
```
text:       h  e  l  l  o  _  w  o  r  l  d
index:      0  1  2  3  4  5  6  7  8  9  10
bonus:     [5, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0]
                              ^ boundary bonus
prev_score: [NO_SCORE × 11]
```

#### i=0, q_char='h' (Query의 첫 문자)
```
j=0, t_char='h': MATCH!
  current_best_score = 0 (i==0이므로)
  current_score[0] = 0 + 5 (bonus) + 10 (MATCH) = 15

j=1~10: no match
  current_score[j] = NO_SCORE

최종:
  current_score = [15, NO_SCORE, NO_SCORE, ..., NO_SCORE]
  prev_score = current_score
```

#### i=1, q_char='w' (Query의 두 번째 문자)

**핵심**: 이전 query 문자('h')가 어디서 매칭되었는지를 기반으로 점수 계산

```
j=0: t_char='h', no match
  current_best_score = NO_SCORE (초기값)

j=1: t_char='e', no match
  current_best_score 업데이트:
    - 현재 best에 gap penalty: NO_SCORE (여전히 NO_SCORE)
    - prev_score[j-1] = prev_score[0] = 15 확인
    - 15 > NO_SCORE이므로 current_best_score = 15

j=2: t_char='l', no match
  current_best_score 업데이트:
    - 현재 best에 gap penalty: 15 + (-2) = 13
    - prev_score[j-1] = prev_score[1] = NO_SCORE
    - 13 > NO_SCORE이므로 current_best_score = 13

j=3: t_char='l', no match
  current_best_score = 13 + (-2) = 11

j=4: t_char='o', no match
  current_best_score = 11 + (-2) = 9

j=5: t_char='_', no match
  current_best_score = 9 + (-2) = 7

j=6: t_char='w', MATCH!
  current_best_score = 7 + (-2) = 5
  current_score[6] = 5 + 3 (boundary bonus) + 10 (MATCH) = 18

j=7~10: no match
  current_score[j] = NO_SCORE

최종:
  current_score = [NO_SCORE, ..., NO_SCORE, 18, NO_SCORE, ...]
  prev_score = current_score
```

#### 결과
```
최종 점수 벡터: [NO_SCORE, ..., 18, ...]
최대 점수: 18 (index 6에서)
```

#### 왜 prev_score[j-1]을 사용하는가?

**이유**: 연속된 문자 매칭의 경로를 추적하기 위해

```
Query: "hw"
Text:  "h e l l o _ w"
        0 1 2 3 4 5 6

이전 문자 'h'가 index 0에서 매칭
현재 문자 'w'가 index 6에서 매칭 시도

current_best_score 계산:
- j=6일 때, prev_score[j-1] = prev_score[5] 확인
- 하지만 'h'는 index 0에서 매칭되었으므로 prev_score[0] = 15
- 이 점수가 j=1부터 gap penalty를 받으며 전파됨:
  j=1: 15
  j=2: 15 + (-2) = 13
  j=3: 13 + (-2) = 11
  j=4: 11 + (-2) = 9
  j=5: 9 + (-2) = 7
  j=6: 7 + (-2) = 5 (이 값이 사용됨)
```

**만약 prev_score[j]를 사용했다면:**
```
j=6일 때, prev_score[6] = NO_SCORE
이전 query 문자가 같은 위치에서 매칭될 수는 없으므로 의미 없음
```

## 구현 평가

### ✅ 잘 구현된 부분

1. **Dynamic Programming 최적화**
   - 2D DP를 1D로 공간 복잡도 최적화
   - O(n×m) 시간, O(n) 공간 (n=text 길이, m=query 길이)

2. **Bonus Score 시스템**
   - 위치별 가중치를 미리 계산하여 효율성 향상
   - 단어 경계, CamelCase, 초기 위치 모두 고려

3. **Gap Penalty 적용**
   - 연속되지 않은 매칭에 패널티 부여
   - 더 긴밀한 매칭에 높은 점수

4. **Case-Insensitive 매칭**
   - `eq_ignore_ascii_case` 사용으로 대소문자 무시
   - 사용자 친화적인 검색 경험

5. **유연한 반환값**
   - `Vec<i32>` 반환으로 각 위치별 점수 제공
   - 필요시 최대값 선택 가능

### ⚠️ 개선 가능한 부분

1. **매개변수 타입**
   ```rust
   // 현재
   pub fn calculate_fzf_score(text: &String, query: &String) -> Vec<i32>

   // 권장
   pub fn calculate_fzf_score(text: &str, query: &str) -> Vec<i32>
   ```
   - `&str`이 더 유연하고 관용적

2. **사용되지 않는 변수**
   ```rust
   let length = text.len();  // Line 14: 사용되지 않음
   ```

3. **헬퍼 함수 추가 고려**
   ```rust
   pub fn get_max_score(text: &str, query: &str) -> i32 {
       calculate_fzf_score(text, query)
           .iter()
           .max()
           .copied()
           .unwrap_or(NO_SCORE)
   }
   ```

## 알고리즘 동작 원리

### Query 첫 문자의 Gap Penalty

**중요**: Query의 첫 문자는 gap penalty 없이 시작합니다.

#### 이유

Query의 첫 문자는 text의 **어느 위치에서든** 매칭을 시작할 수 있어야 합니다:

```rust
query: "hw"
text:  "hello_world"    → h는 index 0에서 시작
text:  "say_hello_world" → h는 index 4에서 시작 (앞부분 무시)
```

만약 첫 문자에도 gap penalty를 적용한다면:

```rust
text: "world_hello"
j=0: score = 0
j=1: score = 0 + GAP = -2
j=2: score = -2 + GAP = -4
...
j=6 ('h' match): score = -12 + bonus + MATCH = 매우 낮은 점수!
```

Text 뒤쪽의 매칭이 불합리하게 낮은 점수를 받게 됩니다.

#### 위치 선호도는 Bonus Score로 처리

Text 앞쪽 위치를 선호하는 것은 **gap penalty가 아니라 bonus score**로 구현:

```rust
if i == 0 {
    score[i] = INITIAL_SCORE;  // 첫 문자 보너스
}
```

### Gap Penalty의 실제 의미

Gap penalty는 **같은 query 내에서 연속되지 않은 매칭**에 적용됩니다:

```rust
query: "ace"
text:  "abcdef"
        ^ ^ ^
        a at 0, c at 2, e at 4
        (b, d를 건너뛰므로 gap penalty)

vs.

text:  "ace"
        ^^^
        a at 0, c at 1, e at 2
        (연속 매칭이므로 gap penalty 최소)
```

## 테스트 케이스

### 1. Exact Match
```rust
text:  "hello"
query: "hello"
expected: 높은 점수 (모든 문자 연속 매칭)
```

### 2. Subsequence Match
```rust
text:  "hello_world"
query: "hw"
expected: 양수 점수 (h at 0, w at 6)
```

### 3. Boundary Bonus
```rust
text1: "hello_world"  (boundary at 6)
text2: "helloworld"   (no boundary)
query: "hw"
expected: score(text1) > score(text2)
```

### 4. CamelCase Bonus
```rust
text:  "helloWorld"
query: "hW"
expected: 양수 점수 (CamelCase 보너스)
```

### 5. No Match
```rust
text:  "hello"
query: "xyz"
expected: 모든 위치 NO_SCORE
```

### 6. Gap Penalty
```rust
text1: "ace"      (연속)
text2: "abcdef"   (gap 있음)
query: "ace"
expected: score(text1) > score(text2)
```

### 7. Initial Position Bonus
```rust
text1: "hello"
text2: "xhello"
query: "h"
expected: score(text1) > score(text2)
```

## 시간/공간 복잡도

- **시간 복잡도**: O(n × m)
  - n = text 길이
  - m = query 길이

- **공간 복잡도**: O(n)
  - bonus_score: O(n)
  - prev_score, current_score: O(n)

## 핵심 알고리즘 개념 요약

### prev_score[j-1] 사용의 중요성

**핵심**: 연속된 문자 매칭의 경로를 올바르게 추적하기 위해 `prev_score[j-1]`을 사용합니다.

```rust
// ✅ 올바른 구현
let score_from_prev_row = prev_score[j - 1];

// ❌ 잘못된 구현
let score_from_prev_row = prev_score[j];
```

**이유**:
- 이전 query 문자가 `text[j-1]`에서 매칭되었다면
- 현재 query 문자는 `text[j]`에서 매칭 (연속된 경로)
- `prev_score[j]`는 이전 query 문자가 현재와 같은 위치에서 매칭된 것을 의미하므로 불가능

### BOUNDARY_SCORE 조정

초기값 `BOUNDARY_SCORE = 1`에서 `BOUNDARY_SCORE = 3`으로 증가:

**이유**: Gap penalty(`-2`)보다 크지 않으면 boundary 매칭의 이점이 상쇄됨

```
Gap penalty: -2 per position
Boundary bonus: 3

→ 1칸 더 떨어진 boundary 매칭이 1점 우위 (3 - 2 = 1)
```

## 결론

현재 구현은 **FZF 알고리즘의 핵심 개념을 올바르게 구현**했습니다:

✅ Dynamic Programming으로 효율적인 계산
✅ Bonus score로 위치별 가중치 적용
✅ Gap penalty로 매칭 품질 구분
✅ Case-insensitive 매칭으로 사용성 향상
✅ Query 첫 문자의 유연한 시작 위치 허용
✅ **prev_score[j-1]을 사용한 올바른 경로 추적**
✅ **적절히 조정된 BOUNDARY_SCORE (gap penalty 대비)**

마이너한 개선사항(매개변수 타입, 헬퍼 함수)을 제외하면, 프로덕션 사용 가능한 수준의 구현입니다.

### 향후 개선 가능한 부분

1. **연속 매칭 보너스**: 연속된 문자 매칭에 추가 점수 부여
2. **사용되지 않는 변수 제거**: `length` 변수
3. **헬퍼 함수 추가**: `get_max_score()` 등
