use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{Datelike, NaiveDateTime, TimeZone, Utc, Weekday};
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use ulid::Ulid;
use uuid::Uuid;

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

fn ulid_to_uuid(ulid_str: &str) -> Result<Uuid, &'static str> {
    let ulid = Ulid::from_string(ulid_str).map_err(|_| "Invalid ULID")?;
    let bytes = ulid.to_bytes();

    // Split the ULID bytes to match the UUID field structure
    let (fields, node) = bytes.split_at(8);
    let (field1, field23) = fields.split_at(4);
    let (field2, field3) = field23.split_at(2);

    let field1 = u32::from_be_bytes(field1.try_into().unwrap());
    let field2 = u16::from_be_bytes(field2.try_into().unwrap());
    let field3 = u16::from_be_bytes(field3.try_into().unwrap());
    let node: [u8; 8] = node.try_into().unwrap();

    Ok(Uuid::from_fields(field1, field2, field3, &node))
}

#[post("/12/ulids")]
async fn convert_ulids_to_uuids(ulids: web::Json<Vec<String>>) -> impl Responder {
    let mut uuids: Vec<String> = ulids
        .into_inner()
        .iter()
        .map(|ulid| ulid_to_uuid(ulid).unwrap().to_string())
        .collect();

    uuids.reverse();
    HttpResponse::Ok().json(uuids)
}

#[post("/12/ulids/{weekday}")]
async fn analyze_ulids(weekday: web::Path<u8>, ulids: web::Json<Vec<String>>) -> impl Responder {
    let mut christmas_eve_count = 0;
    let mut specified_weekday_count = 0;
    let mut future_count = 0;
    let mut lsb_one_count = 0;

    for ulid_str in ulids.into_inner() {
        if let Ok(ulid) = Ulid::from_string(&ulid_str) {
            let timestamp = ulid.timestamp_ms();
            if let Some(datetime) = NaiveDateTime::from_timestamp_opt((timestamp / 1000) as i64, 0)
            {
                let date = Utc.from_utc_datetime(&datetime);

                // Check for christmas eve
                if date.month() == 12 && date.day() == 24 {
                    christmas_eve_count += 1;
                }

                // Check for specified weekday
                if date.weekday() == Weekday::try_from(*weekday).unwrap() {
                    specified_weekday_count += 1;
                }

                // Check for future date
                if Utc::now() < date {
                    future_count += 1;
                }

                // Check if LSB of entropy is 1
                let entropy = &ulid.to_bytes()[6..];
                if entropy[entropy.len() - 1] & 1 == 1 {
                    lsb_one_count += 1;
                }
            }
        }
    }

    HttpResponse::Ok().json(json!({
        "christmas eve": christmas_eve_count,
        "weekday": specified_weekday_count,
        "in the future": future_count,
        "LSB is 1": lsb_one_count
    }))
}

#[cfg(test)]
mod test {
    use actix_web::{http::header, test, App};
    use serde_json::json;
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

    #[actix_web::test]
    async fn test_convert_ulids_to_uuids() {
        let app = test::init_service(App::new().service(convert_ulids_to_uuids)).await;
        let ulids = json!([
            "01BJQ0E1C3Z56ABCD0E11HYX4M",
            "01BJQ0E1C3Z56ABCD0E11HYX5N",
            "01BJQ0E1C3Z56ABCD0E11HYX6Q",
            "01BJQ0E1C3Z56ABCD0E11HYX7R",
            "01BJQ0E1C3Z56ABCD0E11HYX8P"
        ]);

        let req = test::TestRequest::post()
            .uri("/12/ulids")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(ulids.to_string())
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let returned_ulids: Vec<String> = serde_json::from_slice(&body).unwrap();

        let expected_uuids = vec![
            "015cae07-0583-f94c-a5b1-a070431f7516",
            "015cae07-0583-f94c-a5b1-a070431f74f8",
            "015cae07-0583-f94c-a5b1-a070431f74d7",
            "015cae07-0583-f94c-a5b1-a070431f74b5",
            "015cae07-0583-f94c-a5b1-a070431f7494",
        ];

        assert_eq!(returned_ulids, expected_uuids);
    }

    #[actix_web::test]
    async fn test_analyze_ulids() {
        let app = test::init_service(App::new().service(analyze_ulids)).await;

        let ulids = json!([
            "00WEGGF0G0J5HEYXS3D7RWZGV8",
            "76EP4G39R8JD1N8AQNYDVJBRCF",
            "018CJ7KMG0051CDCS3B7BFJ3AK",
            "00Y986KPG0AMGB78RD45E9109K",
            "010451HTG0NYWMPWCEXG6AJ8F2",
            "01HH9SJEG0KY16H81S3N1BMXM4",
            "01HH9SJEG0P9M22Z9VGHH9C8CX",
            "017F8YY0G0NQA16HHC2QT5JD6X",
            "03QCPC7P003V1NND3B3QJW72QJ"
        ]);

        let req = test::TestRequest::post()
            .uri("/12/ulids/5")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(ulids.to_string())
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let expected_result = json!({
            "christmas eve": 3,
            "weekday": 1,
            "in the future": 2,
            "LSB is 1": 5
        });

        assert_eq!(result, expected_result);
    }
}
