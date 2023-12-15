use actix_web::{get, web, web::ServiceConfig, HttpResponse, Responder};
use shuttle_actix_web::ShuttleActixWeb;

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}

#[get("/-1/error")]
async fn negative_one_error() -> impl Responder {
    HttpResponse::InternalServerError()
}

#[get("/1/{first_number_string}/{second_number_string}")]
async fn cube_the_bits(path: web::Path<(String, String)>) -> impl Responder {
    let (first_number_string, second_number_string) = path.into_inner();

    // Attempt to parse to i64
    let first_number = first_number_string.parse::<i64>();
    let second_number = second_number_string.parse::<i64>();

    // Throw if either parse failed
    if first_number.is_err() || second_number.is_err() {
        return HttpResponse::BadRequest().body("Invalid path parameters provided!");
    }

    let xor_result = first_number.unwrap() ^ second_number.unwrap();
    match xor_result.checked_pow(3) {
        Some(value) => HttpResponse::Ok().body(value.to_string()),
        None => HttpResponse::BadRequest().body("Integer overflow!"),
    }
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world)
            .service(negative_one_error)
            .service(cube_the_bits);
    };

    Ok(config.into())
}
