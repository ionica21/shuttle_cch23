use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Reindeer {
    name: String,
    strength: i32,
    speed: Option<f64>,
    height: Option<i32>,
    antler_width: Option<i32>,
    snow_magic_power: Option<i32>,
    favorite_food: Option<String>,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten_yesterday: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ContestResponse {
    fastest: Option<String>,
    tallest: Option<String>,
    magician: Option<String>,
    consumer: Option<String>,
}

#[post("/4/strength")]
async fn reindeer_strength(reindeers: web::Json<Vec<Reindeer>>) -> impl Responder {
    let strength_option = reindeers.iter().fold(Some(0_i64), |acc, reindeer| {
        acc.and_then(|acc_value| acc_value.checked_add(i64::from(reindeer.strength)))
    });
    match strength_option {
        Some(strength) => HttpResponse::Ok().body(strength.to_string()),
        None => HttpResponse::BadRequest().body("Overflow occurred in strength calculation"),
    }
}

#[post("/4/contest")]
async fn reindeer_contest(reindeers: web::Json<Vec<Reindeer>>) -> impl Responder {
    let fastest = reindeers
        .iter()
        .filter(|reindeer| reindeer.speed.is_some())
        .max_by(|reindeer_a, reindeer_b| {
            reindeer_a
                .speed
                .unwrap()
                .partial_cmp(&reindeer_b.speed.unwrap())
                .unwrap()
        });
    let tallest = reindeers
        .iter()
        .filter(|reindeer| reindeer.height.is_some())
        .max_by_key(|reindeer| reindeer.height.unwrap());
    let magician = reindeers
        .iter()
        .filter(|reindeer| reindeer.snow_magic_power.is_some())
        .max_by_key(|reindeer| reindeer.snow_magic_power.unwrap());
    let consumer = reindeers
        .iter()
        .filter(|reindeer| reindeer.candies_eaten_yesterday.is_some())
        .max_by_key(|reindeer| reindeer.candies_eaten_yesterday.unwrap());
    HttpResponse::Ok().json(ContestResponse {
        fastest: match fastest {
            Some(reindeer) => Some(format!(
                "Speeding past the finish line with a strength of {} is {}",
                reindeer.strength, reindeer.name
            )),
            None => None,
        },
        tallest: match tallest {
            Some(reindeer) => Some(format!(
                "{} is standing tall with his {} cm wide antlers",
                reindeer.name,
                reindeer.antler_width.as_ref().unwrap_or(&0_i32)
            )),
            None => None,
        },
        magician: match magician {
            Some(reindeer) => Some(format!(
                "{} could blast you away with a snow magic power of {}",
                reindeer.name,
                reindeer.snow_magic_power.as_ref().unwrap_or(&0_i32)
            )),
            None => None,
        },
        consumer: match consumer {
            Some(reindeer) => Some(format!(
                "{} ate lots of candies, but also some {}",
                reindeer.name,
                reindeer.favorite_food.as_ref().unwrap_or(&"".to_string())
            )),
            None => None,
        },
    })
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use serde_json;

    use super::*;

    #[actix_web::test]
    async fn test_reindeer_strength() {
        let app = test::init_service(App::new().service(reindeer_strength)).await;

        let req = test::TestRequest::post()
            .uri("/4/strength")
            .set_json(&vec![
                Reindeer {
                    name: "Dasher".to_string(),
                    strength: 5,
                    speed: None,
                    height: None,
                    antler_width: None,
                    snow_magic_power: None,
                    favorite_food: None,
                    candies_eaten_yesterday: None,
                },
                Reindeer {
                    name: "Dancer".to_string(),
                    strength: 6,
                    speed: None,
                    height: None,
                    antler_width: None,
                    snow_magic_power: None,
                    favorite_food: None,
                    candies_eaten_yesterday: None,
                },
                Reindeer {
                    name: "Prancer".to_string(),
                    strength: 4,
                    speed: None,
                    height: None,
                    antler_width: None,
                    snow_magic_power: None,
                    favorite_food: None,
                    candies_eaten_yesterday: None,
                },
                Reindeer {
                    name: "Vixen".to_string(),
                    strength: 7,
                    speed: None,
                    height: None,
                    antler_width: None,
                    snow_magic_power: None,
                    favorite_food: None,
                    candies_eaten_yesterday: None,
                },
            ])
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body = test::read_body(res).await;
        assert_eq!(res_body, "22");
    }

    #[actix_web::test]
    async fn test_reindeer_contest() {
        let app = test::init_service(App::new().service(reindeer_contest)).await;

        let req = test::TestRequest::post()
            .uri("/4/contest")
            .set_json(&vec![
                Reindeer {
                    name: "Dasher".to_string(),
                    strength: 5,
                    speed: Some(50.4),
                    height: Some(80),
                    antler_width: Some(36),
                    snow_magic_power: Some(9001),
                    favorite_food: Some("hay".to_string()),
                    candies_eaten_yesterday: Some(2),
                },
                Reindeer {
                    name: "Dancer".to_string(),
                    strength: 6,
                    speed: Some(48.2),
                    height: Some(65),
                    antler_width: Some(37),
                    snow_magic_power: Some(4004),
                    favorite_food: Some("grass".to_string()),
                    candies_eaten_yesterday: Some(5),
                },
            ])
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");

        let content_response: ContestResponse =
            serde_json::from_str(&res_body).expect("Failed to deserialize response");

        assert_eq!(
            content_response,
            ContestResponse {
                fastest: Some(
                    "Speeding past the finish line with a strength of 5 is Dasher".to_string()
                ),
                tallest: Some("Dasher is standing tall with his 36 cm wide antlers".to_string()),
                magician: Some(
                    "Dasher could blast you away with a snow magic power of 9001".to_string()
                ),
                consumer: Some("Dancer ate lots of candies, but also some grass".to_string()),
            }
        );
    }
}
