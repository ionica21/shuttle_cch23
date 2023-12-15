use actix_web::{web, HttpResponse, Responder};

pub async fn cube_the_bits(path: web::Path<String>) -> impl Responder {
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
