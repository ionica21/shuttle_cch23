use actix_web::{post, HttpResponse, Responder};
use std::collections::{HashMap, VecDeque};

#[post("/22/integers")]
async fn find_unpaired_integer(body: String) -> impl Responder {
    let mut unpaired = 0u64;

    for line in body.lines() {
        if let Ok(num) = line.parse::<u64>() {
            unpaired ^= num;
        }
    }

    HttpResponse::Ok().body("🎁".repeat(unpaired as usize))
}

#[derive(Clone, Copy, Debug)]
struct Star {
    x: i32,
    y: i32,
    z: i32,
}

fn distance(a: Star, b: Star) -> f32 {
    let dx = (b.x - a.x) as f32;
    let dy = (b.y - a.y) as f32;
    let dz = (b.z - a.z) as f32;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[post("/22/rocket")]
async fn find_path_and_distance(body: String) -> impl Responder {
    let mut lines = body.lines();

    let n: usize = lines.next().unwrap().parse().unwrap();
    let mut stars = Vec::new();
    for _ in 0..n {
        let coords: Vec<i32> = lines
            .next()
            .unwrap()
            .split_whitespace()
            .map(|num| num.parse().unwrap())
            .collect();
        stars.push(Star {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        });
    }

    let k: usize = lines.next().unwrap().parse().unwrap();
    let mut graph = HashMap::new();
    for _ in 0..k {
        let portal: Vec<usize> = lines
            .next()
            .unwrap()
            .split_whitespace()
            .map(|num| num.parse().unwrap())
            .collect();
        graph
            .entry(portal[0])
            .or_insert_with(Vec::new)
            .push(portal[1]);
        graph
            .entry(portal[1])
            .or_insert_with(Vec::new)
            .push(portal[0]);
    }

    let mut queue = VecDeque::new();
    let mut visited = vec![false; n];
    let mut prev = vec![None; n];
    queue.push_back(0);
    visited[0] = true;

    while let Some(current) = queue.pop_front() {
        for &neighbor in &graph[&current] {
            if !visited[neighbor] {
                queue.push_back(neighbor);
                visited[neighbor] = true;
                prev[neighbor] = Some(current);
            }
        }
    }

    let mut path = Vec::new();
    let mut current = n - 1;
    while let Some(p) = prev[current] {
        path.push(current);
        current = p;
    }
    path.push(0);
    path.reverse();

    let mut total_distance = 0f32;
    for window in path.windows(2) {
        total_distance += distance(stars[window[0]], stars[window[1]]);
    }

    HttpResponse::Ok().body(format!("{} {:.3}", path.len() - 1, total_distance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test, web, App};
    use std::fs;

    #[actix_web::test]
    async fn test_find_unpaired_integer() {
        let mut app = test::init_service(App::new().service(find_unpaired_integer)).await;

        let req = test::TestRequest::post()
            .uri("/22/integers")
            .insert_header((header::CONTENT_TYPE, "text/plain"))
            .set_payload("888\n77\n888\n22\n77\n")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        assert_eq!(body_str, "🎁".repeat(22));
    }

    #[actix_web::test]
    async fn test_find_unpaired_integer_from_large_file() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::PayloadConfig::new(1024 * 1024))
                .service(find_unpaired_integer),
        )
        .await;

        // Read from the file
        let file_contents =
            fs::read_to_string("static/numbers.txt").expect("Failed to read from file");

        let req = test::TestRequest::post()
            .uri("/22/integers")
            .insert_header((header::CONTENT_TYPE, "text/plain"))
            .set_payload(file_contents)
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        // Replace this with the expected outcome based on the file's contents
        assert_eq!(body_str, "🎁".repeat(120003).as_str());
    }

    #[actix_web::test]
    async fn test_find_path_and_distance() {
        let mut app = test::init_service(App::new().service(find_path_and_distance)).await;

        let test_data = "5\n0 1 0\n-2 2 3\n3 -3 -5\n1 1 5\n4 3 5\n4\n0 1\n2 4\n3 4\n1 2\n";
        let req = test::TestRequest::post()
            .uri("/22/rocket")
            .insert_header((header::CONTENT_TYPE, "text/plain"))
            .set_payload(test_data)
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        let body_str = String::from_utf8(body.to_vec()).expect("Failed to convert to UTF-8");

        assert_eq!(body_str, "3 26.123");
    }
}
