use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn hello_world() -> &'static str {
    "Hello World!"
}

#[get("/-1/error")]
pub async fn negative_one_error() -> impl Responder {
    HttpResponse::InternalServerError()
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    use super::*;

    #[actix_web::test]
    async fn test_hello_world() {
        let app = test::init_service(App::new().service(hello_world)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body = test::read_body(res).await;
        assert_eq!(res_body, "Hello World!");
    }

    #[actix_web::test]
    async fn test_negative_one_error() {
        let app = test::init_service(App::new().service(negative_one_error)).await;
        let req = test::TestRequest::get().uri("/-1/error").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_server_error());
    }
}
