use actix_web::{get, post, web, HttpResponse, Responder};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

lazy_static! {
    static ref STORE: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

#[post("/12/save/{string}")]
async fn save_string(string: web::Path<String>) -> impl Responder {
    STORE
        .lock()
        .unwrap()
        .insert(string.into_inner(), Instant::now());
    HttpResponse::Ok().body("Saved!")
}

#[get("/12/load/{string}")]
async fn load_string(string: web::Path<String>) -> impl Responder {
    if let Some(saved_at) = STORE.lock().unwrap().get(string.as_str()) {
        let elapsed = Instant::now().duration_since(*saved_at);
        return HttpResponse::Ok().body(elapsed.as_secs().to_string());
    }
    HttpResponse::BadRequest().body("Key not found in store!")
}

#[cfg(test)]
mod test {
    use actix_web::{test, App};
    use std::time::Duration;
    use tokio::time::sleep;

    use super::*;

    #[actix_web::test]
    async fn test_save_and_load_string() {
        let app = test::init_service(App::new().service(save_string).service(load_string)).await;

        let req = test::TestRequest::post()
            .uri("/12/save/packet20231212")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Wait for 2 seconds
        sleep(Duration::from_secs(2)).await;

        let req = test::TestRequest::get()
            .uri("/12/load/packet20231212")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
        let body = test::read_body(res).await;
        assert_eq!(&body, "2");

        // Wait for another 2 seconds
        sleep(Duration::from_secs(2)).await;

        let req = test::TestRequest::get()
            .uri("/12/load/packet20231212")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
        let body = test::read_body(res).await;
        assert_eq!(&body, "4");

        let req = test::TestRequest::post()
            .uri("/12/save/packet20231212")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let req = test::TestRequest::get()
            .uri("/12/load/packet20231212")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
        let body = test::read_body(res).await;
        assert_eq!(&body, "0");
    }
}
