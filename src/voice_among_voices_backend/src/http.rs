use ic_cdk::api::data_certificate;
use ic_http_certification::{
    cel::create_cel_expr, utils::add_skip_certification_header, CelExpression, DefaultCelBuilder,
    DefaultCelExpression, DefaultResponseCertification, HttpCertification, HttpCertificationPath,
    HttpCertificationTree, HttpCertificationTreeEntry, HttpRequest, HttpResponse, StatusCode,
};
use serde_bytes::ByteBuf;

use crate::{images::generate_nft_image, storage::voice_nodes::get_stored_voice_nodes};

pub fn http_request(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url().split('?').collect();
    let path = parts[0];

    // Match paths like "/123"
    let re = regex::Regex::new(r"^/([1-9]|[1-9][0-9]|[1-2][0-9]{2}|3[0-5][0-9]|360)$").unwrap();

    if let Some(caps) = re.captures(path) {
        let nft_id: usize = caps[1].parse().unwrap();
        let nodes = get_stored_voice_nodes();
        // match generate_nft_image(&nodes, nft_id as u32) {
        //     Ok(image) => {
        // let cel_expr = DefaultCelBuilder::full_certification()
        //     .with_request_headers(
        //         req.headers
        //             .iter()
        //             .map(|(header, _)| header.as_str())
        //             .collect::<Vec<_>>(),
        //     )
        //     .with_response_certification(
        //         DefaultResponseCertification::certified_response_headers(vec![
        //             "Content-Type",
        //             "Content-Length",
        //             "Cache-Control",
        //         ]),
        //     )
        //     .build();
        let image = generate_nft_image(&nodes, nft_id as u32);
        let cel_expr = DefaultCelBuilder::skip_certification();

        let response_headers = vec![
            ("Content-Type".to_string(), "image/png".to_string()),
            ("Content-Length".to_string(), image.len().to_string()),
            ("Cache-Control".to_string(), "no-cache".to_string()),
            ("IC-CertificateExpression".to_string(), cel_expr.to_string()),
        ];

        let mut response = HttpResponse::builder()
            .with_body(ByteBuf::from(image).to_vec())
            .with_headers(response_headers)
            .with_status_code(StatusCode::from_u16(200).unwrap())
            .with_upgrade(false)
            .build();

        add_skip_certification_header(data_certificate().unwrap(), &mut response);

        // let req_url = format!("/{}", nft_id);
        // let certification = HttpCertification::skip();

        // let path = HttpCertificationPath::exact(&req_url);

        // let mut http_certification_tree = HttpCertificationTree::default();

        // let entry = HttpCertificationTreeEntry::new(&path, &certification);

        // http_certification_tree.insert(&entry);

        // let witness = http_certification_tree.witness(&entry, &req_url);

        response
        // }
        // Err(_) => HttpResponse::builder()
        //     .with_status_code(StatusCode::from_u16(500).unwrap())
        //     .with_body(ByteBuf::from("Image generation failed").to_vec())
        //     .build(),
    } else {
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(404).unwrap())
            .with_body(ByteBuf::from("Not found").to_vec())
            .build()
    }
}

// pub fn security_headers(
//     integrity_hashes: Vec<String>,
//     maybe_related_origins: Option<Vec<String>>,
// ) -> Vec<(String, String)> {
//     vec![
//         ("X-Frame-Options".to_string(), "DENY".to_string()),
//         ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
//         (
//             "Content-Security-Policy".to_string(),
//             content_security_policy_header(integrity_hashes, maybe_related_origins),
//         ),
//         (
//             "Strict-Transport-Security".to_string(),
//             "max-age=31536000 ; includeSubDomains".to_string(),
//         ),
//         ("Referrer-Policy".to_string(), "same-origin".to_string()),
//     ]
// }
