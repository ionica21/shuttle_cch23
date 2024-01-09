use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct Paginate {
    offset: usize,
    limit: usize,
}

#[post("/5")]
async fn slice_names(
    paginate: web::Query<Paginate>,
    body: web::Json<Vec<String>>,
) -> impl Responder {
    let start = paginate.offset;
    let end = start + paginate.limit;
    let sliced_names = &body[start..end.min(body.len())];

    HttpResponse::Ok().json(sliced_names)
}

#[cfg(test)]
mod test {
    use actix_web::{http::header, test, App};

    use super::*;

    #[actix_web::test]
    async fn test_slice_names() {
        let mut app = test::init_service(App::new().service(slice_names)).await;

        let req = test::TestRequest::post()
            .uri("/5?offset=3&limit=5")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(r#"[ "Ava", "Caleb", "Mia", "Owen", "Lily", "Ethan", "Zoe", "Nolan", "Harper", "Lucas", "Stella", "Mason", "Olivia" ]"#)
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        assert_eq!(body_str, r#"["Owen","Lily","Ethan","Zoe","Nolan"]"#);
    }
}
