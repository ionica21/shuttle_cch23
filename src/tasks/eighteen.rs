use crate::AppState;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Executor, Row};

#[post("/18/reset")]
async fn reset_advanced(state: web::Data<AppState>) -> impl Responder {
    let res = state
        .pool
        .execute(
            "
            DROP TABLE IF EXISTS regions;
            DROP TABLE IF EXISTS orders;

            CREATE TABLE regions (
              id INT PRIMARY KEY,
              name VARCHAR(50)
            );

            CREATE TABLE orders (
              id INT PRIMARY KEY,
              region_id INT,
              gift_name VARCHAR(50),
              quantity INT
            );",
        )
        .await;

    if let Ok(_) = res {
        HttpResponse::Ok().body("Orders reset!")
    } else {
        HttpResponse::InternalServerError().body("Failed to reset orders!")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Region {
    id: i64,
    name: String,
}

#[post("/18/regions")]
async fn add_regions(
    state: web::Data<AppState>,
    regions: web::Json<Vec<Region>>,
) -> Result<HttpResponse, Error> {
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    for region in regions.into_inner() {
        sqlx::query("INSERT INTO regions (id, name) VALUES ($1, $2)")
            .bind(region.id)
            .bind(region.name)
            .execute(&mut *transaction)
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;
    }

    transaction
        .commit()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body("Successfully added regions!"))
}

#[get("/18/regions/total")]
async fn total_regions(state: web::Data<AppState>) -> impl Responder {
    if let Ok(rows) = sqlx::query(
        "SELECT name, SUM(quantity) FROM regions
            INNER JOIN orders ON regions.id = orders.region_id
            GROUP BY name;",
    )
    .fetch_all(&state.pool)
    .await
    {
        let mut result: Vec<Value> = rows
            .into_iter()
            .map(|row| {
                let name: String = row.get(0);
                let count: i64 = row.get(1);
                json!({"region": name, "total": count})
            })
            .collect();
        result.sort_by(|a, b| {
            a["region"]
                .as_str()
                .unwrap_or("")
                .cmp(&b["region"].as_str().unwrap_or(""))
        });

        HttpResponse::Ok().json(result)
    } else {
        HttpResponse::InternalServerError().body("Failed to get total regions")
    }
}

#[get("/18/regions/top_list/{number}")]
async fn top_list(state: web::Data<AppState>, number: web::Path<i32>) -> impl Responder {
    match sqlx::query_as::<_, (String, Option<Vec<String>>)>("
        WITH ranked_gifts AS (
            SELECT
                regions.name AS name,
                orders.gift_name,
                SUM(orders.quantity) AS total_quantity,
                ROW_NUMBER() OVER(PARTITION BY regions.id ORDER BY SUM(orders.quantity) DESC, orders.gift_name) AS rank
            FROM orders
            INNER JOIN regions ON regions.id = orders.region_id
            GROUP BY regions.id, orders.gift_name
        )
        SELECT
            regions.name,
            ARRAY_AGG(gift_name ORDER BY total_quantity DESC, gift_name) FILTER (WHERE rank <= $1 AND gift_name IS NOT NULL) AS top_gifts
        FROM regions
        LEFT JOIN ranked_gifts ON regions.name = ranked_gifts.name
        GROUP BY regions.name
        ORDER BY regions.name")
        .bind(number.into_inner())
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => {
            let result: Vec<Value> = rows
                .into_iter()
                .map(|(region_name, top_gifts)| {
                    json!({"region": region_name, "top_gifts": top_gifts.unwrap_or_else(Vec::new)})
                })
                .collect();

            HttpResponse::Ok().json(result)
        }
        Err(err) => {
            println!("err: {}", err);
            HttpResponse::InternalServerError().body("Failed to fetch top list")
        },
    }
}

#[cfg(test)]
mod test {
    use crate::tasks;
    use actix_web::web::Bytes;
    use actix_web::{test, App};
    use serde_json::json;
    use serial_test::serial;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::{Arc, Mutex};
    use tokio::fs;
    use toml::Table;

    use super::*;

    async fn set_up_sql() -> web::Data<AppState> {
        let secrets_contents = fs::read_to_string("Secrets.dev.toml").await.unwrap();
        let secrets = secrets_contents.parse::<Table>().unwrap();
        let password = secrets
            .get("PG_PASSWORD")
            .and_then(toml::Value::as_str)
            .unwrap();
        let port = secrets
            .get("PG_PORT")
            .and_then(toml::Value::as_str)
            .unwrap();
        let database_url = format!(
            "postgres://postgres:{}@localhost:{}/postgres",
            password, port
        );
        let pool = PgPoolOptions::new().connect(&database_url).await.unwrap();
        web::Data::new(AppState {
            pool,
            rooms: Default::default(),
            view_count: Arc::new(Mutex::new(0)),
        })
    }

    #[actix_web::test]
    #[serial]
    async fn test_total_orders_per_region() {
        let state = set_up_sql().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .service(reset_advanced)
                .service(add_regions)
                .route("/18/orders", web::post().to(tasks::thirteen::add_orders))
                .service(total_regions),
        )
        .await;

        // Reset state
        let req = test::TestRequest::post().uri("/18/reset").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Add regions
        let regions_data = json!([
            {"id":1,"name":"North Pole"},
            {"id":2,"name":"Europe"},
            {"id":3,"name":"North America"},
            {"id":4,"name":"South America"},
            {"id":5,"name":"Africa"},
            {"id":6,"name":"Asia"},
            {"id":7,"name":"Oceania"}
        ]);
        let req = test::TestRequest::post()
            .uri("/18/regions")
            .set_json(&regions_data)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Add orders
        let orders_data = json!([
            {"id":1,"region_id":2,"gift_name":"Board Game","quantity":5},
            {"id":2,"region_id":2,"gift_name":"Origami Set","quantity":8},
            {"id":3,"region_id":3,"gift_name":"Action Figure","quantity":12},
            {"id":4,"region_id":4,"gift_name":"Teddy Bear","quantity":10},
            {"id":5,"region_id":2,"gift_name":"Yarn Ball","quantity":6},
            {"id":6,"region_id":3,"gift_name":"Art Set","quantity":3},
            {"id":7,"region_id":5,"gift_name":"Robot Lego Kit","quantity":5},
            {"id":8,"region_id":6,"gift_name":"Drone","quantity":9}
        ]);
        let req = test::TestRequest::post()
            .uri("/18/orders")
            .set_json(&orders_data)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Get total orders per region
        let req = test::TestRequest::get()
            .uri("/18/regions/total")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let expected_response = json!([
           {"region":"Africa","total":5},
            {"region":"Asia","total":9},
            {"region":"Europe","total":19},
            {"region":"North America","total":15},
            {"region":"South America","total":10}
        ])
        .to_string();
        assert_eq!(body, Bytes::from(expected_response));
    }

    #[actix_web::test]
    #[serial]
    async fn test_top_list_per_region() {
        let state = set_up_sql().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .service(reset_advanced)
                .service(add_regions)
                .route("/18/orders", web::post().to(tasks::thirteen::add_orders))
                .service(top_list),
        )
        .await;

        // Reset state
        let req = test::TestRequest::post().uri("/18/reset").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Add regions
        let regions_data = json!([
            {"id":1,"name":"North Pole"},
            {"id":2,"name":"South Pole"},
            {"id":3,"name":"Kiribati"},
            {"id":4,"name":"Baker Island"}
        ]);
        let req = test::TestRequest::post()
            .uri("/18/regions")
            .set_json(&regions_data)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Add orders
        let orders_data = json!([
            {"id":1,"region_id":2,"gift_name":"Toy Train","quantity":5},
            {"id":2,"region_id":2,"gift_name":"Toy Train","quantity":3},
            {"id":3,"region_id":2,"gift_name":"Doll","quantity":8},
            {"id":4,"region_id":3,"gift_name":"Toy Train","quantity":3},
            {"id":5,"region_id":2,"gift_name":"Teddy Bear","quantity":6},
            {"id":6,"region_id":3,"gift_name":"Action Figure","quantity":12},
            {"id":7,"region_id":4,"gift_name":"Board Game","quantity":10},
            {"id":8,"region_id":3,"gift_name":"Teddy Bear","quantity":1},
            {"id":9,"region_id":3,"gift_name":"Teddy Bear","quantity":2}
        ]);
        let req = test::TestRequest::post()
            .uri("/18/orders")
            .set_json(&orders_data)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Get top list per region
        let req = test::TestRequest::get()
            .uri("/18/regions/top_list/2")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let expected_response = json!([
            {"region":"Baker Island","top_gifts":["Board Game"]},
            {"region":"Kiribati","top_gifts":["Action Figure","Teddy Bear"]},
            {"region":"North Pole","top_gifts":[]},
            {"region":"South Pole","top_gifts":["Doll","Toy Train"]}
        ])
        .to_string();
        assert_eq!(body, Bytes::from(expected_response));
    }
}
