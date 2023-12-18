use actix_web::{get, web, HttpResponse, Responder};

#[get("/8/weight/{pokemon_id}")]
pub async fn pokemon_weight(pokemon_id: web::Path<i64>) -> impl Responder {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemon_res = rustemon::pokemon::pokemon::get_by_id(*pokemon_id, &rustemon_client).await;
    if let Ok(pokemon) = pokemon_res {
        let weight_kg = pokemon.weight as f64 / 10_f64;
        HttpResponse::Ok().body(weight_kg.to_string())
    } else {
        HttpResponse::BadRequest().body("Bad request")
    }
}

#[cfg(test)]
mod test {
    use actix_web::{test, App};

    use super::*;

    #[actix_web::test]
    async fn test_pokemon_weight() {
        let app = test::init_service(App::new().service(pokemon_weight)).await;

        let req = test::TestRequest::get().uri("/8/weight/25").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");
        assert_eq!(res_body, "60");
    }
}
