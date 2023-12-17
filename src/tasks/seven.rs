use actix_web::{get, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::string::FromUtf8Error;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct BakeData {
    recipe: HashMap<String, u32>,
    pantry: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BakeResponse {
    cookies: u32,
    pantry: HashMap<String, u32>,
}

fn decode_cookies_as_string(req: &HttpRequest) -> Result<String, FromUtf8Error> {
    let recipe_cookie = req.cookie("recipe").unwrap();
    let recipe_bytes = general_purpose::STANDARD
        .decode(recipe_cookie.value())
        .unwrap_or(vec![]);
    return String::from_utf8(recipe_bytes);
}

#[get("/7/decode")]
async fn decode_recipe(req: HttpRequest) -> impl Responder {
    if let Ok(recipe_string) = decode_cookies_as_string(&req) {
        return HttpResponse::Ok().body(recipe_string);
    }
    HttpResponse::BadRequest().body("Invalid Cookie header")
}

fn can_bake_cookie(bake_data: &BakeData) -> bool {
    let mut can_bake = true;
    for (ingredient, recipe_amount) in bake_data.recipe.iter() {
        if let Some(pantry_amount) = bake_data.pantry.get(ingredient) {
            if pantry_amount < recipe_amount {
                can_bake = false;
                break;
            }
        } else {
            can_bake = false;
            break;
        }
    }
    return can_bake;
}

fn bake_cookie(bake_data: &mut BakeData) {
    for (ingredient, recipe_amount) in bake_data.recipe.iter() {
        bake_data.pantry.insert(
            ingredient.to_string(),
            bake_data.pantry.get(ingredient).unwrap() - recipe_amount,
        );
    }
}

#[get("/7/bake")]
async fn bake_recipe(req: HttpRequest) -> impl Responder {
    if let Ok(bake_data_string) = decode_cookies_as_string(&req) {
        if let Ok(mut bake_data) = serde_json::from_str::<BakeData>(bake_data_string.as_str()) {
            let mut cookies_baked = 0;
            while can_bake_cookie(&bake_data) {
                bake_cookie(&mut bake_data);
                cookies_baked += 1;
            }
            HttpResponse::Ok().json(BakeResponse {
                cookies: cookies_baked,
                pantry: bake_data.pantry,
            })
        } else {
            HttpResponse::BadRequest().body("Invalid bake data")
        }
    } else {
        HttpResponse::BadRequest().body("Invalid Cookie header")
    }
}

#[cfg(test)]
mod test {
    use actix_web::cookie::Cookie;
    use actix_web::{test, App};
    use serde_json::{json, Value};

    use super::*;

    #[actix_web::test]
    async fn test_decode() {
        let app = test::init_service(App::new().service(decode_recipe)).await;

        let req = test::TestRequest::get()
            .cookie(Cookie::new(
                "recipe",
                "eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==",
            ))
            .uri("/7/decode")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");
        assert_eq!(res_body, "{\"flour\":100,\"chocolate chips\":20}");
    }

    #[actix_web::test]
    async fn test_bake() {
        let app = test::init_service(App::new().service(bake_recipe)).await;

        let req = test::TestRequest::get()
            .cookie(Cookie::new(
                "recipe",
                "eyJyZWNpcGUiOnsiZmxvdXIiOjk1LCJzdWdhciI6NTAsImJ1dHRlciI6MzAsImJha2luZyBwb3dkZXIiOjEwLCJjaG9jb2xhdGUgY2hpcHMiOjUwfSwicGFudHJ5Ijp7ImZsb3VyIjozODUsInN1Z2FyIjo1MDcsImJ1dHRlciI6MjEyMiwiYmFraW5nIHBvd2RlciI6ODY1LCJjaG9jb2xhdGUgY2hpcHMiOjQ1N319",
            ))
            .uri("/7/bake")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");
        assert_eq!(
            serde_json::from_str::<Value>(&res_body).unwrap(),
            json!({
              "cookies": 4,
              "pantry": {
                "flour": 5,
                "sugar": 307,
                "butter": 2002,
                "baking powder": 825,
                "chocolate chips": 257
              }
            })
        );
    }

    #[actix_web::test]
    async fn test_bake_questionable() {
        let app = test::init_service(App::new().service(bake_recipe)).await;

        let req = test::TestRequest::get()
            .cookie(Cookie::new(
                "recipe",
                "eyJyZWNpcGUiOnsic2xpbWUiOjl9LCJwYW50cnkiOnsiY29iYmxlc3RvbmUiOjY0LCJzdGljayI6IDR9fQ==",
            ))
            .uri("/7/bake")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");
        assert_eq!(
            serde_json::from_str::<Value>(&res_body).unwrap(),
            json!({
              "cookies": 0,
              "pantry": {
                "cobblestone": 64,
                "stick": 4
              }
            })
        );
    }
}
