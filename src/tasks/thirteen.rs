use crate::AppState;
use actix_web::{error, get, web, Error, HttpResponse};
use sqlx::Row;

#[get("/13/sql")]
async fn sql(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let row = sqlx::query("SELECT 20231213;")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| error::ErrorBadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().body(row.get::<i32, _>(0).to_string()))
}

#[cfg(test)]
mod test {
    use actix_web::{test, App};
    use sqlx::postgres::PgPoolOptions;
    use tokio::fs;
    use toml::Table;

    use super::*;

    #[actix_web::test]
    async fn test_sql() {
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
        let state = web::Data::new(AppState { pool });
        let app = test::init_service(App::new().app_data(state).service(sql)).await;

        let req = test::TestRequest::get().uri("/13/sql").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "20231213");
    }
}
