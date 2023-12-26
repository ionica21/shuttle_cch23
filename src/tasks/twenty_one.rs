use actix_web::{get, web, HttpResponse, Responder};
use s2::{cellid::CellID, latlng::LatLng};

#[get("/21/coords/{binary}")]
async fn get_coords(binary: web::Path<String>) -> impl Responder {
    let cell_id = u64::from_str_radix(&binary, 2).unwrap();
    let s2_cell_id = CellID(cell_id);
    let lat_lng = LatLng::from(s2_cell_id);

    HttpResponse::Ok().body(format_lat_lng_to_dms(lat_lng))
}

fn format_lat_lng_to_dms(lat_lng: LatLng) -> String {
    let lat_dms = convert_to_dms(lat_lng.lat.deg(), true);
    let lng_dms = convert_to_dms(lat_lng.lng.deg(), false);

    format!("{} {}", lat_dms, lng_dms)
}

fn convert_to_dms(deg: f64, is_latitude: bool) -> String {
    let (degrees, direction) = if deg < 0.0 {
        (-deg, if is_latitude { "S" } else { "W" })
    } else {
        (deg, if is_latitude { "N" } else { "E" })
    };

    let degrees_int = degrees as i32;
    let minutes = (degrees - degrees_int as f64) * 60.0;
    let seconds = (minutes - minutes as i32 as f64) * 60.0;

    format!(
        "{:02}°{:02}'{:06.3}''{}",
        degrees_int, minutes as i32, seconds, direction
    )
}

#[cfg(test)]
mod test {
    use actix_web::{test, App};

    use super::*;

    #[actix_web::test]
    async fn test_get_coords_1() {
        let mut app = test::init_service(App::new().service(get_coords)).await;

        let req = test::TestRequest::get()
            .uri("/21/coords/0100111110010011000110011001010101011111000010100011110001011011")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "83°39'54.324''N 30°37'40.584''W"
        );
    }

    #[actix_web::test]
    async fn test_get_coords_2() {
        let mut app = test::init_service(App::new().service(get_coords)).await;

        let req = test::TestRequest::get()
            .uri("/21/coords/0010000111110000011111100000111010111100000100111101111011000101")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let body = test::read_body(res).await;
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "18°54'55.944''S 47°31'17.976''E"
        );
    }
}
