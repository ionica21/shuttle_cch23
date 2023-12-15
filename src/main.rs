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

async fn cube_the_bits(path: web::Path<String>) -> impl Responder {
    // Attempt to parse to i64
    let numbers_result: Result<Vec<i64>, _> = path
        .split("/")
        .filter(|item| !item.is_empty())
        .map(|item| item.parse::<i64>())
        .collect();


    // Throw if parse failed
    if numbers_result.is_err() {
        return HttpResponse::BadRequest().body("Invalid path parameters provided!");
    }

    let xor_result = numbers_result.unwrap().iter().fold(0, |acc, x| acc ^ x);
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
            .route("/1/{tail:.*}", web::get().to(cube_the_bits));
    };

    Ok(config.into())
}
