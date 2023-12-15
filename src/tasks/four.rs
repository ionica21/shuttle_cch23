use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
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

#[derive(Serialize)]
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
