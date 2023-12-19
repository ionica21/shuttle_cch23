use actix_web::{post, web, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(
    source = "<html>\n  <head>\n    <title>CCH23 Day 14</title>\n  </head>\n  <body>\n    {{ content }}\n  </body>\n</html>",
    ext = "html",
    escape = "none"
)]
struct UnsafeTemplate {
    content: String,
}

#[derive(Template)]
#[template(
    source = "<html>\n  <head>\n    <title>CCH23 Day 14</title>\n  </head>\n  <body>\n    {{ content }}\n  </body>\n</html>",
    ext = "html",
    escape = "html"
)]
struct SafeTemplate {
    content: String,
}

#[derive(Deserialize)]
struct HtmlInput {
    content: String,
}

#[post("/14/unsafe")]
async fn unsafe_endpoint(input: web::Json<HtmlInput>) -> impl Responder {
    let template = UnsafeTemplate {
        content: input.content.clone(),
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

#[post("/14/safe")]
async fn safe_endpoint(input: web::Json<HtmlInput>) -> impl Responder {
    let template = SafeTemplate {
        content: input.content.clone(),
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

#[cfg(test)]
mod test {
    use actix_web::{http::header, test, App};
    use serde_json::json;

    use super::*;

    #[actix_web::test]
    async fn test_unsafe_endpoint() {
        let app = test::init_service(App::new().service(unsafe_endpoint)).await;

        let input_data = json!({ "content": "<h1>Welcome to the North Pole!</h1>" });
        let req = test::TestRequest::post()
            .uri("/14/unsafe")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(input_data.to_string())
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let expected_html = "<html>\n  <head>\n    <title>CCH23 Day 14</title>\n  </head>\n  <body>\n    <h1>Welcome to the North Pole!</h1>\n  </body>\n</html>";

        assert_eq!(body, expected_html);
    }

    #[actix_web::test]
    async fn test_safe_endpoint() {
        let app = test::init_service(App::new().service(safe_endpoint)).await;

        let input_data = json!({ "content": "<script>alert(\"XSS Attack!\")</script>" });
        let req = test::TestRequest::post()
            .uri("/14/safe")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .set_payload(input_data.to_string())
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let expected_html = "<html>\n  <head>\n    <title>CCH23 Day 14</title>\n  </head>\n  <body>\n    &lt;script&gt;alert(&quot;XSS Attack!&quot;)&lt;/script&gt;\n  </body>\n</html>";

        assert_eq!(body, expected_html);
    }
}
