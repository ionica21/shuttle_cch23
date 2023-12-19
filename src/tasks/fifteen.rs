use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};

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
}
