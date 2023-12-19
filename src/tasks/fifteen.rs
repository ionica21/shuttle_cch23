use actix_web::{post, web, HttpResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
struct PasswordInput {
    input: String,
}

#[derive(Serialize, Deserialize)]
struct ValidationResult {
    result: String,
}

#[post("/15/nice")]
async fn validate_password(input: web::Json<PasswordInput>) -> HttpResponse {
    if is_nice(&input.input) {
        HttpResponse::Ok().json(ValidationResult {
            result: "nice".to_string(),
        })
    } else {
        HttpResponse::BadRequest().json(ValidationResult {
            result: "naughty".to_string(),
        })
    }
}

fn is_nice(s: &str) -> bool {
    let vowels = "aeiouy";
    let forbidden = ["ab", "cd", "pq", "xy"];

    let vowel_count = s.chars().filter(|&c| vowels.contains(c)).count();
    let has_double = s
        .chars()
        .zip(s.chars().skip(1))
        .any(|(a, b)| a == b && a.is_alphabetic() && b.is_alphabetic());
    let has_forbidden = forbidden.iter().any(|&f| s.contains(f));

    vowel_count >= 3 && has_double && !has_forbidden
}

#[derive(Serialize, Deserialize)]
struct GameInput {
    input: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GameResult {
    result: String,
    reason: String,
}

#[post("/15/game")]
async fn game(input: web::Json<GameInput>) -> HttpResponse {
    match validate_string(&input.input) {
        Ok(_) => HttpResponse::Ok().json(GameResult {
            result: "nice".to_string(),
            reason: "that's a nice password".to_string(),
        }),
        Err(reason) => match &reason[..] {
            "8 chars" => HttpResponse::BadRequest().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "more types of chars" => HttpResponse::BadRequest().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "55555" => HttpResponse::BadRequest().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "math is hard" => HttpResponse::BadRequest().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "not joyful enough" => HttpResponse::NotAcceptable().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "illegal: no sandwich" => HttpResponse::UnavailableForLegalReasons().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "outranged" => HttpResponse::RangeNotSatisfiable().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "😳" => HttpResponse::UpgradeRequired().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            "not a coffee brewer" => HttpResponse::ImATeapot().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
            _ => HttpResponse::BadRequest().json(GameResult {
                result: "naughty".to_string(),
                reason,
            }),
        },
    }
}

fn validate_string(s: &str) -> Result<(), String> {
    validate_rule_1(s)?;
    validate_rule_2(s)?;
    validate_rule_3(s)?;
    validate_rule_4(s)?;
    validate_rule_5(s)?;
    validate_rule_6(s)?;
    validate_rule_7(s)?;
    validate_rule_8(s)?;
    validate_rule_9(s)?;

    Ok(())
}

fn validate_rule_1(s: &str) -> Result<(), String> {
    if s.len() >= 8 {
        Ok(())
    } else {
        Err("8 chars".to_string())
    }
}

fn validate_rule_2(s: &str) -> Result<(), String> {
    let has_uppercase = s.chars().any(|c| c.is_uppercase());
    let has_lowercase = s.chars().any(|c| c.is_lowercase());
    let has_digit = s.chars().any(|c| c.is_digit(10));

    if has_uppercase && has_lowercase && has_digit {
        Ok(())
    } else {
        Err("more types of chars".to_string())
    }
}

fn validate_rule_3(s: &str) -> Result<(), String> {
    let digit_count = s.chars().filter(|c| c.is_digit(10)).count();
    if digit_count >= 5 {
        Ok(())
    } else {
        Err("55555".to_string())
    }
}

fn validate_rule_4(s: &str) -> Result<(), String> {
    // Replace potential Unicode escape sequences with a placeholder
    let cleaned_string = Regex::new(r"\\u\{[0-9A-Fa-f]+\}")
        .unwrap()
        .replace_all(s, "X");

    let digit_regex = Regex::new(r"\d+").unwrap();
    let total: u32 = digit_regex
        .find_iter(&cleaned_string)
        .filter_map(|item| item.as_str().parse::<u32>().ok())
        .sum();

    if total == 2023 {
        Ok(())
    } else {
        Err("math is hard".to_string())
    }
}

fn validate_rule_5(s: &str) -> Result<(), String> {
    let mut found_j = false;
    let mut found_o = false;
    let mut found_y = false;

    for c in s.chars() {
        match c {
            'j' => {
                if found_o || found_y {
                    return Err("not joyful enough".to_string());
                }
                found_j = true;
            }
            'o' => {
                if !found_j || found_y {
                    return Err("not joyful enough".to_string());
                }
                found_o = true;
            }
            'y' => {
                if !found_j || !found_o {
                    return Err("not joyful enough".to_string());
                }
                if found_y {
                    // Second 'y' found, which is not allowed
                    return Err("not joyful enough".to_string());
                }
                found_y = true;
            }
            _ => {}
        }
    }

    if found_j && found_o && found_y {
        Ok(())
    } else {
        Err("not joyful enough".to_string())
    }
}

fn validate_rule_6(s: &str) -> Result<(), String> {
    let chars: Vec<char> = s.chars().collect();
    for i in 1..chars.len() - 1 {
        if chars[i - 1].is_alphabetic()
            && chars[i].is_alphabetic()
            && chars[i + 1].is_alphabetic()
            && chars[i - 1] == chars[i + 1]
        {
            return Ok(());
        }
    }
    Err("illegal: no sandwich".to_string())
}

fn validate_rule_7(s: &str) -> Result<(), String> {
    if s.chars().any(|c| ('\u{2980}'..='\u{2BFF}').contains(&c)) {
        Ok(())
    } else {
        Err("outranged".to_string())
    }
}

fn validate_rule_8(s: &str) -> Result<(), String> {
    // Unicode range for common emojis
    let emoji_pattern = Regex::new(
        r"[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F900}-\u{1F9FF}]",
    )
    .unwrap();

    if emoji_pattern.is_match(s) {
        Ok(())
    } else {
        Err("😳".to_string())
    }
}

fn validate_rule_9(s: &str) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    if format!("{:x}", result).ends_with("a") {
        Ok(())
    } else {
        Err("not a coffee brewer".to_string())
    }
}

#[cfg(test)]
mod test {
    use actix_web::{test, App};

    use super::*;
    #[actix_web::test]
    async fn test_nice_password() {
        let app = test::init_service(App::new().service(validate_password)).await;

        let req = test::TestRequest::post()
            .uri("/15/nice")
            .set_json(&PasswordInput {
                input: "hello there".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let res: ValidationResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.result, "nice");
    }

    #[actix_web::test]
    async fn test_naughty_password() {
        let app = test::init_service(App::new().service(validate_password)).await;

        let req = test::TestRequest::post()
            .uri("/15/nice")
            .set_json(&PasswordInput {
                input: "abcd".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_client_error());

        let body = test::read_body(res).await;
        let res: ValidationResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.result, "naughty");
    }

    #[actix_web::test]
    async fn test_invalid_payload() {
        let app = test::init_service(App::new().service(validate_password)).await;

        let req = test::TestRequest::post()
            .uri("/15/nice")
            .set_payload("{Grinch? GRINCH!}")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_game_endpoint() {
        let app = test::init_service(App::new().service(game)).await;

        // Test case 1: "password"
        let req = test::TestRequest::post()
            .uri("/15/game")
            .set_json(&GameInput {
                input: "password".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_client_error());

        let body = test::read_body(res).await;
        let result: GameResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            result,
            GameResult {
                result: "naughty".to_string(),
                reason: "more types of chars".to_string()
            }
        );

        // Test case 2: "Password12345"
        let req = test::TestRequest::post()
            .uri("/15/game")
            .set_json(&GameInput {
                input: "Password12345".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_client_error());

        let body = test::read_body(res).await;
        let result: GameResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            result,
            GameResult {
                result: "naughty".to_string(),
                reason: "math is hard".to_string()
            }
        );

        // Test case 3: "23jPassword2000y"
        let req = test::TestRequest::post()
            .uri("/15/game")
            .set_json(&GameInput {
                input: "23jPassword2000y".to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_client_error());

        let body = test::read_body(res).await;
        let result: GameResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            result,
            GameResult {
                result: "naughty".to_string(),
                reason: "illegal: no sandwich".to_string()
            }
        );
    }
}
