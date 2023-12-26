use actix_web::{post, web, HttpResponse, Responder};
use futures::StreamExt;
use git2::{Commit, Repository};
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

#[post("/20/cookie")]
async fn find_cookie(mut payload: web::Payload) -> impl Responder {
    let mut archive_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        archive_data.extend_from_slice(&chunk);
    }

    let cursor = Cursor::new(archive_data);
    let mut archive = Archive::new(cursor);
    let temp_dir = tempfile::tempdir().unwrap();
    archive.unpack(temp_dir.path()).unwrap();

    match Repository::open(temp_dir.path()) {
        Ok(repo) => {
            if let Some(commit_info) = find_commit_with_cookie(&repo) {
                HttpResponse::Ok().body(format!("{} {}", commit_info.0, commit_info.1))
            } else {
                HttpResponse::BadRequest().body("Failed to find commit with cookie!")
            }
        }
        Err(_err) => HttpResponse::BadRequest().body("Failed to read repository details!"),
    }
}

fn find_commit_with_cookie(repo: &Repository) -> Option<(String, String)> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_ref("refs/heads/christmas").unwrap();

    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        if check_commit_for_cookie(&commit, repo) {
            return Some((
                commit.author().name().unwrap().to_string(),
                commit.id().to_string(),
            ));
        }
    }

    None
}

fn check_commit_for_cookie(commit: &Commit, repo: &Repository) -> bool {
    let tree = commit.tree().unwrap();
    check_tree_for_cookie(&tree, repo)
}

fn check_tree_for_cookie(tree: &git2::Tree, repo: &Repository) -> bool {
    for entry in tree.iter() {
        match entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let sub_tree = repo.find_tree(entry.id()).unwrap();
                if check_tree_for_cookie(&sub_tree, repo) {
                    return true;
                }
            }
            Some(git2::ObjectType::Blob) => {
                if entry.name().unwrap() == "santa.txt" {
                    let blob = repo.find_blob(entry.id()).unwrap();
                    let content = std::str::from_utf8(blob.content()).unwrap();
                    if content.contains("COOKIE") {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::fs::File;
    use std::io::Read;

    use super::*;

    fn read_test_tar_file(location: &str) -> Vec<u8> {
        let mut file = File::open(location).expect("Test file not found");
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
            .set_payload(read_test_tar_file("static/northpole20231220.tar"))
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
            .set_payload(read_test_tar_file("static/northpole20231220.tar"))
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "1196282");
    }

    #[actix_web::test]
    async fn test_find_cookie() {
        let mut app = test::init_service(App::new().service(find_cookie)).await;

        let req = test::TestRequest::post()
            .uri("/20/cookie")
            .set_payload(read_test_tar_file("static/cookiejar.tar"))
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(body, "Grinch 71dfab551a1958b35b7436c54b7455dcec99a12c");
    }
}
