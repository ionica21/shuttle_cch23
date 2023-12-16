mod tasks;

use actix_web::{web, web::ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(tasks::negative_one::hello_world)
            .service(tasks::negative_one::negative_one_error)
            .route("/1/{tail:.*}", web::get().to(tasks::one::cube_the_bits))
            .service(tasks::four::reindeer_strength)
            .service(tasks::four::reindeer_contest);
    };

    Ok(config.into())
}
