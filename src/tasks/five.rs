use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct Paginate {
    offset: Option<usize>,
    limit: Option<usize>,
    split: Option<usize>,
}

#[post("/5")]
async fn slice_names(
    paginate: web::Query<Paginate>,
    body: web::Json<Vec<String>>,
) -> impl Responder {
    let offset = paginate.offset.unwrap_or(0);
    let limit = paginate.limit.unwrap_or(body.len());
    let split = paginate.split;

    let end = (offset + limit).min(body.len());
    let sliced = &body[offset..end];

    match split {
        Some(split_size) => HttpResponse::Ok().json(
            sliced
                .chunks(split_size)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<Vec<String>>>(),
        ),
        None => HttpResponse::Ok().json(sliced),
    }
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

    #[actix_web::test]
    async fn test_slice_names_with_split() {
        let mut app = test::init_service(App::new().service(slice_names)).await;

        let req = test::TestRequest::post()
            .uri("/5?split=4")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(r#"[ "Ava", "Caleb", "Mia", "Owen", "Lily", "Ethan", "Zoe", "Nolan", "Harper", "Lucas", "Stella", "Mason", "Olivia" ]"#)
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        assert_eq!(
            body_str,
            r#"[["Ava","Caleb","Mia","Owen"],["Lily","Ethan","Zoe","Nolan"],["Harper","Lucas","Stella","Mason"],["Olivia"]]"#
        );
    }

    #[actix_web::test]
    async fn test_slice_names_with_offset_and_split() {
        let mut app = test::init_service(App::new().service(slice_names)).await;

        let req = test::TestRequest::post()
            .uri("/5?offset=5&split=2")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(r#"[ "Ava", "Caleb", "Mia", "Owen", "Lily", "Ethan", "Zoe", "Nolan", "Harper", "Lucas", "Stella", "Mason", "Olivia" ]"#)
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        assert_eq!(
            body_str,
            r#"[["Ethan","Zoe"],["Nolan","Harper"],["Lucas","Stella"],["Mason","Olivia"]]"#
        );
    }
}
