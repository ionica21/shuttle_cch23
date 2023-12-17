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

#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    use super::*;

    #[actix_web::test]
    async fn test_cube_the_bits() {
        let app =
            test::init_service(App::new().route("/1/{tail:.*}", web::get().to(cube_the_bits)))
                .await;

        let req = test::TestRequest::get().uri("/1/4/8").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body = test::read_body(res).await;
        assert_eq!(res_body, "1728");
    }

    #[actix_web::test]
    async fn test_cube_the_bits_sled_id() {
        let app =
            test::init_service(App::new().route("/1/{tail:.*}", web::get().to(cube_the_bits)))
                .await;

        let req = test::TestRequest::get().uri("/1/4/5/8/10").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body = test::read_body(res).await;
        assert_eq!(res_body, "27");
    }
}
