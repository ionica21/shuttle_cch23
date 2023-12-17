use actix_web::{post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ElfCounts {
    elf: i32,
    #[serde(rename = "elf on a shelf")]
    elf_on_a_shelf: i32,
    #[serde(rename = "shelf with no elf on it")]
    shelf_with_no_elf_on_it: i32,
}

fn clean_text(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation() || *c == ' ')
        .collect::<String>()
}

// Both the .match method and regex do not handle overlapping matches
fn count_overlapping_matches(text: &str, pattern: &str) -> (usize, Vec<usize>) {
    let mut count = 0;
    let mut start = 0;

    // Save where the matches were found so that we can check for a shelf with no elf
    let mut match_indices: Vec<usize> = vec![];

    while let Some(match_index) = text[start..].find(pattern) {
        count += 1;
        match_indices.push(start + match_index);
        start += match_index + 1; // Move just past the last found character
    }

    (count, match_indices)
}

#[post("/6")]
pub async fn count_elf(body: String) -> impl Responder {
    let cleaned_body = clean_text(&body);
    let (elf_count, _) = count_overlapping_matches(&cleaned_body, "elf");
    let (elf_on_a_shelf_count, elf_on_a_shelf_indices) =
        count_overlapping_matches(&cleaned_body, "elf on a shelf");

    // Counting "shelf" occurrences that are not preceded by "elf on a ".
    let shelf_with_no_elf_count = elf_on_a_shelf_indices
        .into_iter()
        .filter(|index| cleaned_body[..*index].ends_with("elf on a "))
        .count();

    HttpResponse::Ok().json(ElfCounts {
        elf: elf_count as i32,
        elf_on_a_shelf: elf_on_a_shelf_count as i32,
        shelf_with_no_elf_on_it: shelf_with_no_elf_count as i32,
    })
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use serde_json;

    use super::*;

    #[actix_web::test]
    async fn test_count_elf_simple() {
        let app = test::init_service(App::new().service(count_elf)).await;

        let req = test::TestRequest::post()
            .uri("/6")
            .set_payload(
                "The mischievous elf peeked out from behind the toy workshop,
                 and another elf joined in the festive dance.
                 Look, there is also an elf on that shelf!",
            )
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");

        let elf_counts: ElfCounts =
            serde_json::from_str(&res_body).expect("Failed to deserialize response");
        assert_eq!(
            elf_counts,
            ElfCounts {
                elf: 4,
                elf_on_a_shelf: 0,
                shelf_with_no_elf_on_it: 1,
            }
        );
    }

    #[actix_web::test]
    async fn test_count_elf_complex() {
        let app = test::init_service(App::new().service(count_elf)).await;

        let req = test::TestRequest::post()
            .uri("/6")
            .set_payload(
                "there is an elf on a shelf on an elf.
      there is also another shelf in Belfast.",
            )
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let res_body_bytes = test::read_body(res).await;
        let res_body = String::from_utf8(res_body_bytes.to_vec())
            .expect("Failed to convert response to string");

        assert_eq!(
            res_body,
            "{\"elf\":5,\"elf on a shelf\":1,\"shelf with no elf on it\":1}"
        );

        let elf_counts: ElfCounts =
            serde_json::from_str(&res_body).expect("Failed to deserialize response");
        assert_eq!(
            elf_counts,
            ElfCounts {
                elf: 5,
                elf_on_a_shelf: 1,
                shelf_with_no_elf_on_it: 1,
            }
        );
    }
}
