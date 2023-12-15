use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn hello_world() -> &'static str {
    "Hello World!"
}

#[get("/-1/error")]
pub async fn negative_one_error() -> impl Responder {
    HttpResponse::InternalServerError()
}
