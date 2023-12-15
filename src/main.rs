mod task_negative_one;
mod task_one;

use actix_web::{web, web::ServiceConfig, Responder};
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(task_negative_one::hello_world)
            .service(task_negative_one::negative_one_error)
            .route("/1/{tail:.*}", web::get().to(task_one::cube_the_bits));
    };

    Ok(config.into())
}
