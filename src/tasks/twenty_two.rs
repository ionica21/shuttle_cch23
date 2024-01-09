use actix_web::{post, HttpResponse, Responder};

#[post("/22/integers")]
async fn find_unpaired_integer(body: String) -> impl Responder {
    let mut unpaired = 0u64;

    for line in body.lines() {
        if let Ok(num) = line.parse::<u64>() {
            unpaired ^= num;
        }
    }

    HttpResponse::Ok().body("🎁".repeat(unpaired as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test, web, App};
    use std::fs;

    #[actix_web::test]
    async fn test_find_unpaired_integer() {
        let mut app = test::init_service(App::new().service(find_unpaired_integer)).await;

        let req = test::TestRequest::post()
            .uri("/22/integers")
            .insert_header((header::CONTENT_TYPE, "text/plain"))
            .set_payload("888\n77\n888\n22\n77\n")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        assert_eq!(body_str, "🎁".repeat(22));
    }

    #[actix_web::test]
    async fn test_find_unpaired_integer_from_large_file() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::PayloadConfig::new(1024 * 1024))
                .service(find_unpaired_integer),
        )
        .await;

        // Read from the file
        let file_contents =
            fs::read_to_string("static/numbers.txt").expect("Failed to read from file");

        let req = test::TestRequest::post()
            .uri("/22/integers")
            .insert_header((header::CONTENT_TYPE, "text/plain"))
            .set_payload(file_contents)
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        // Replace this with the expected outcome based on the file's contents
        assert_eq!(body_str, "🎁".repeat(120003).as_str());
    }
}
