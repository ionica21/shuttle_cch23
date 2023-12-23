mod tasks;

use actix_web::{web, web::ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_shared_db;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.PG_PASSWORD}@localhost:{secrets.PG_PORT}/postgres"
    )]
    pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let state = web::Data::new(AppState { pool });
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(state)
            .service(tasks::negative_one::hello_world)
            .service(tasks::negative_one::negative_one_error)
            .route("/1/{tail:.*}", web::get().to(tasks::one::cube_the_bits))
            .service(tasks::four::reindeer_strength)
            .service(tasks::four::reindeer_contest)
            .service(tasks::six::count_elf)
            .service(tasks::seven::decode_recipe)
            .service(tasks::seven::bake_recipe)
            .service(tasks::eight::pokemon_weight)
            .service(tasks::eight::pokemon_drop)
            .service(tasks::eleven::assets)
            .service(tasks::eleven::count_red_pixels)
            .service(tasks::twelve::save_string)
            .service(tasks::twelve::load_string)
            .service(tasks::twelve::convert_ulids_to_uuids)
            .service(tasks::twelve::analyze_ulids)
            .service(tasks::thirteen::sql)
            .service(tasks::thirteen::reset)
            .route("/13/orders", web::post().to(tasks::thirteen::add_orders))
            .service(tasks::thirteen::total_orders)
            .service(tasks::thirteen::most_popular_gift)
            .service(tasks::fourteen::unsafe_endpoint)
            .service(tasks::fourteen::safe_endpoint)
            .service(tasks::fifteen::validate_password)
            .service(tasks::fifteen::game)
            .service(tasks::eighteen::reset_advanced)
            .route("/18/orders", web::post().to(tasks::thirteen::add_orders))
            .service(tasks::eighteen::add_regions)
            .service(tasks::eighteen::total_regions)
            .service(tasks::eighteen::top_list);
    };

    Ok(config.into())
}
