use crate::AppState;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Executor, Row};

#[get("/13/sql")]
async fn sql(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let row = sqlx::query("SELECT 20231213;")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| error::ErrorBadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().body(row.get::<i32, _>(0).to_string()))
}

#[post("/13/reset")]
async fn reset(state: web::Data<AppState>) -> impl Responder {
    let res = state
        .pool
        .execute(
            "DROP TABLE IF EXISTS orders;
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
struct Order {
    id: i64,
    region_id: i64,
    gift_name: String,
    quantity: i64,
}

#[post("13/orders")]
async fn add_orders(
    state: web::Data<AppState>,
    orders: web::Json<Vec<Order>>,
) -> Result<HttpResponse, Error> {
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    for order in orders.into_inner() {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name)
        .bind(order.quantity)
        .execute(&mut *transaction)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;
    }

    transaction
        .commit()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body("Successfully added orders!"))
}

#[get("/13/orders/total")]
async fn total_orders(state: web::Data<AppState>) -> impl Responder {
    if let Ok(row) = sqlx::query("SELECT SUM(quantity) FROM orders;")
        .fetch_one(&state.pool)
        .await
    {
        let count = row.get::<i64, _>(0);
        HttpResponse::Ok().json(json!({"total": count}))
    } else {
        HttpResponse::InternalServerError().body("Failed to get total orders")
    }
}

#[cfg(test)]
mod test {
    use actix_web::{test, App};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
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
        web::Data::new(AppState { pool })
    }

    #[actix_web::test]
    async fn test_sql() {
        let state = set_up_sql().await;
        let app = test::init_service(App::new().app_data(state).service(sql)).await;

        let req = test::TestRequest::get().uri("/13/sql").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "20231213");
    }

    #[actix_web::test]
    async fn test_orders_process() {
        let state = set_up_sql().await;
        let app = test::init_service(
            App::new()
                .app_data(state)
                .service(reset)
                .service(add_orders)
                .service(total_orders),
        )
        .await;

        // Reset state
        let req = test::TestRequest::post().uri("/13/reset").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Post orders
        let orders_data = json!([
            {"id":1,"region_id":2,"gift_name":"Toy Train","quantity":5},
            {"id":2,"region_id":2,"gift_name":"Doll","quantity":8},
            {"id":3,"region_id":3,"gift_name":"Action Figure","quantity":12},
            {"id":4,"region_id":4,"gift_name":"Board Game","quantity":10},
            {"id":5,"region_id":2,"gift_name":"Teddy Bear","quantity":6},
            {"id":6,"region_id":3,"gift_name":"Toy Train","quantity":3}
        ]);
        let req = test::TestRequest::post()
            .uri("/13/orders")
            .set_json(&orders_data)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // Get total orders
        let req = test::TestRequest::get()
            .uri("/13/orders/total")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "{\"total\":44}");
    }
}
