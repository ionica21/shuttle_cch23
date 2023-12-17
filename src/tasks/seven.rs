use actix_web::{get, HttpRequest, HttpResponse, Responder};

use base64::{engine::general_purpose, Engine as _};

#[get("/7/decode")]
async fn decode_recipe(req: HttpRequest) -> impl Responder {
    if let Some(recipe_cookie) = req.cookie("recipe") {
        let recipe_bytes = general_purpose::STANDARD
            .decode(recipe_cookie.value())
            .unwrap();
        if let Ok(recipe_string) = String::from_utf8(recipe_bytes) {
            return HttpResponse::Ok().body(recipe_string);
        }
    }
    HttpResponse::BadRequest().body("Invalid Cookie header")
}

#[cfg(test)]
mod test {
    use actix_web::cookie::Cookie;
    use actix_web::{test, App};

    use super::*;

    #[actix_web::test]
    async fn test_count_elf_simple() {
        let app = test::init_service(App::new().service(decode_recipe)).await;

        let req = test::TestRequest::get()
            .cookie(Cookie::new(
                "recipe",
                "eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==",
            ))
            .uri("/7/decode")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");
        assert_eq!(res_body, "{\"flour\":100,\"chocolate chips\":20}");
    }
}
