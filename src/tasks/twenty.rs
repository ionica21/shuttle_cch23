use actix_web::{post, web, HttpResponse, Responder};
use futures::StreamExt;
use std::io::Cursor;
use tar::Archive;

#[post("/20/archive_files")]
async fn archive_files(mut payload: web::Payload) -> impl Responder {
    let mut archive_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        archive_data.extend_from_slice(&chunk);
    }

    let cursor = Cursor::new(archive_data);
    let mut archive = Archive::new(cursor);
    let file_count = archive.entries().unwrap().count();

    HttpResponse::Ok().body(file_count.to_string())
}

#[post("/20/archive_files_size")]
async fn archive_files_size(mut payload: web::Payload) -> impl Responder {
    let mut archive_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        archive_data.extend_from_slice(&chunk);
    }

    let cursor = Cursor::new(archive_data);
    let mut archive = tar::Archive::new(cursor);
    let total_size = archive
        .entries()
        .unwrap()
        .map(|e| e.unwrap().header().size().unwrap())
        .sum::<u64>();

    HttpResponse::Ok().body(total_size.to_string())
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::fs::File;
    use std::io::Read;

    use super::*;

    fn read_test_tar_file() -> Vec<u8> {
        let mut file = File::open("static/northpole20231220.tar").expect("Test file not found");
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .expect("Failed to read test file");
        data
    }

    #[actix_web::test]
    async fn test_count_files() {
        let mut app = test::init_service(App::new().service(archive_files)).await;

        let req = test::TestRequest::post()
            .uri("/20/archive_files")
            .set_payload(read_test_tar_file())
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "6");
    }

    #[actix_web::test]
    async fn test_sum_file_sizes() {
        let mut app = test::init_service(App::new().service(archive_files_size)).await;

        let req = test::TestRequest::post()
            .uri("/20/archive_files_size")
            .set_payload(read_test_tar_file())
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "1196282");
    }
}
