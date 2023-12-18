use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use image::io::Reader as ImageReader;
use image::{GenericImageView, Pixel};
use std::io::Cursor;

#[get("/11/assets/{file_name}")]
async fn assets(file_name: web::Path<String>) -> impl Responder {
    NamedFile::open_async(format!("static/{}", *file_name)).await
}

#[post("/11/red_pixels")]
async fn count_red_pixels(mut payload: Multipart) -> Result<HttpResponse, Error> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or("");

        if name == "image" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk?;
                data.extend_from_slice(&chunk);
            }

            let img = ImageReader::new(Cursor::new(data))
                .with_guessed_format()?
                .decode()
                .map_err(|e| error::ErrorInternalServerError(e))?;

            let red_pixels = img
                .pixels()
                .filter(|(_, _, rgba)| {
                    let channels = rgba.channels();
                    // r > (g + b)
                    channels[0] as u16 > (channels[1] as u16 + channels[2] as u16)
                })
                .count();

            return Ok(HttpResponse::Ok().body(red_pixels.to_string()));
        }
    }

    Ok(HttpResponse::BadRequest().body("No image field in the request"))
}

#[cfg(test)]
mod test {
    use actix_web::{http, test, App};
    use http::header::CONTENT_TYPE;

    use super::*;

    #[actix_web::test]
    async fn test_assets() {
        let app = test::init_service(App::new().service(assets)).await;

        let req = test::TestRequest::get()
            .uri("/11/assets/decoration.png")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let content_type = res
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok().map(String::from));
        let body = test::read_body(res).await;
        let content_length = body.len();
        assert_eq!(content_type, Some("image/png".to_owned()));
        assert_eq!(content_length, 787297);
    }

    #[actix_web::test]
    async fn test_count_red_pixels() {
        let mut app = test::init_service(App::new().service(count_red_pixels)).await;

        let file_path = "static/decoration.png";
        let file = std::fs::read(file_path).expect("Unable to read file");

        let boundary = "boundary";
        let mut request_body = Vec::new();

        request_body.extend_from_slice(
            format!(
                "--{}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"decoration.png\"\r\nContent-Type: image/png\r\n\r\n",
                boundary
            ).as_bytes()
        );

        request_body.extend_from_slice(&file);
        request_body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

        let req = test::TestRequest::post()
            .uri("/11/red_pixels")
            .insert_header((
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            ))
            .set_payload(request_body)
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let res_body = test::read_body(res).await;
        assert_eq!(res_body, "73034");
    }
}
