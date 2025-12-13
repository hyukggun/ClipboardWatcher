use std::i32;

const INITIAL_SCORE: i32 = 5;
const BOUNDARY_SCORE: i32 = 3;
const CAMEL_CASE_SCORE: i32 = 2;
const MATCH_SCORE: i32 = 10;
const GAP_SCORE: i32 = -2;

const NO_SCORE: i32 = -10000;

// Calculate the bonus score for the text
fn calculate_bonus_score(text: &String) -> Vec<i32> {
    let mut score = vec![0; text.len()];
    let length = text.len();

    for (i, char) in text.chars().enumerate() {
        if i == 0 {
            score[i] = INITIAL_SCORE;
        }
        else {
            let prev_char = text.chars().nth(i - 1).unwrap();
            if matches!(prev_char, '/' | '_' | '-' | '.' | ' ') {
                score[i] = BOUNDARY_SCORE;
            }

            if prev_char.is_lowercase() && char.is_uppercase() {
                score[i] = CAMEL_CASE_SCORE;
            }
        }
    }
    score
}

pub fn calculate_fzf_score(text: &String, query: &String) -> Vec<i32> {
    let bonus_score = calculate_bonus_score(text);

    // 이전 행의 점수 저장
    let mut prev_score: Vec<i32> = vec![NO_SCORE; text.len()];

    for (i, q_char) in query.chars().enumerate() {
        // 현재 행의 점수 저장
        let mut current_score = vec![NO_SCORE; text.len()];
        let mut current_best_score = NO_SCORE;
        for (j, t_char) in text.chars().enumerate() {
            // 첫 행이 아닌 경우
            if i > 0 {
                // 이전 최고 점수가 있는 경우
                if current_best_score > NO_SCORE {
                    current_best_score += GAP_SCORE;
                }

                // 이전 행의 최고 점수가 있는 경우
                if j > 0 {
                    let score_from_prev_row = prev_score[j - 1];
                    if score_from_prev_row > current_best_score {
                        current_best_score = score_from_prev_row;
                    }
                }
            } else {
                current_best_score = 0;
            }

            if q_char.eq_ignore_ascii_case(&t_char) {
                // 현재 행에 대한 점수 = 이전 행의 최고 점수 + 보너스 점수 + 매칭 점수
                current_score[j] = current_best_score + bonus_score[j] + MATCH_SCORE;
            }
        }
        prev_score = current_score;
    }
    prev_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let text = String::from("hello");
        let query = String::from("hello");
        let scores = calculate_fzf_score(&text, &query);

        // Exact match should have a high score
        let max_score = scores.iter().max().unwrap();
        assert!(*max_score > 0, "Exact match should have positive score");
    }

    #[test]
    fn test_subsequence_match() {
        let text = String::from("hello_world");
        let query = String::from("hw");
        let scores = calculate_fzf_score(&text, &query);

        // Should match 'h' at index 0 and 'w' at index 6
        let max_score = scores.iter().max().unwrap();
        assert!(*max_score > 0, "Subsequence match should have positive score");
    }

    #[test]
    fn test_boundary_bonus() {
        let text1 = String::from("hello_world");
        let text2 = String::from("helloworld");
        let query = String::from("hw");

        let scores1 = calculate_fzf_score(&text1, &query);
        let scores2 = calculate_fzf_score(&text2, &query);

        let max_score1 = scores1.iter().max().unwrap();
        let max_score2 = scores2.iter().max().unwrap();

        println!("text1: 'hello_world' -> max_score: {}", max_score1);
        println!("text2: 'helloworld' -> max_score: {}", max_score2);
        println!("scores1: {:?}", scores1);
        println!("scores2: {:?}", scores2);

        // Boundary match should score higher
        assert!(max_score1 > max_score2,
            "Match at word boundary should score higher than in middle of word. text1={}, text2={}", max_score1, max_score2);
    }

    #[test]
    fn test_camel_case_bonus() {
        let text = String::from("helloWorld");
        let query = String::from("hW");
        let scores = calculate_fzf_score(&text, &query);

        // Should match 'h' at 0 and 'W' at 5 with camelCase bonus
        let max_score = scores.iter().max().unwrap();
        assert!(*max_score > 0, "CamelCase match should have positive score");
    }

    #[test]
    fn test_no_match() {
        let text = String::from("hello");
        let query = String::from("xyz");
        let scores = calculate_fzf_score(&text, &query);

        // No match should result in all NO_SCORE
        assert!(scores.iter().all(|&s| s == NO_SCORE),
            "Non-matching query should have all NO_SCORE");
    }

    #[test]
    fn test_gap_penalty() {
        let text = String::from("abcdef");
        let query = String::from("ace");
        let scores = calculate_fzf_score(&text, &query);

        // Should match but with gap penalties
        let max_score = scores.iter().max().unwrap();
        assert!(*max_score > 0, "Should match with gap penalty");

        // Compare with consecutive match
        let text2 = String::from("ace");
        let scores2 = calculate_fzf_score(&text2, &query);
        let max_score2 = scores2.iter().max().unwrap();

        assert!(max_score2 > max_score,
            "Consecutive match should score higher than match with gaps");
    }

    #[test]
    fn test_initial_position_bonus() {
        let text1 = String::from("hello");
        let text2 = String::from("xhello");
        let query = String::from("h");

        let scores1 = calculate_fzf_score(&text1, &query);
        let scores2 = calculate_fzf_score(&text2, &query);

        let max_score1 = scores1.iter().max().unwrap();
        let max_score2 = scores2.iter().max().unwrap();

        // Initial position should have bonus
        assert!(max_score1 > max_score2,
            "Match at initial position should score higher");
    }
}