use actix_web::{
    delete, get,
    http::header,
    options, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use sanitize_filename::sanitize;
use std::{fs::File, io::Read, path::Path};

const IMG_STORAGE_PATH: &str = ".\\img-test-storage";
const IMG_FORMAT: &str = "jpg";

#[get("/img/{filename}")]
async fn get_img(info: web::Path<(String,)>) -> impl Responder {
    let filename = format!("{}{}{}", sanitize(&info.0), ".", IMG_FORMAT);
    let file_path = Path::new(IMG_STORAGE_PATH).join(&filename);

    // Open file
    let Ok(mut file) = File::open(&file_path) else {
        return HttpResponse::NotFound().body("Picture not found");
    };

    // Read file content
    let mut img_content = Vec::new();
    let Ok(_) = file.read_to_end(&mut img_content) else {
        return HttpResponse::NoContent().body("Picture file corrupt");
    };

    // Send file back
    return HttpResponse::Ok()
        .append_header(header::ContentType::jpeg())
        .append_header(header::ContentDisposition::attachment(filename))
        .body(img_content);
}

#[options("/{tail:.*}")]
async fn handle_options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .finish()
}

#[post("/upload")]
async fn upload_img() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .finish()
}

#[delete("/delete/{file}")]
async fn delete_img() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_img)
            .service(handle_options)
            .service(upload_img)
            .service(delete_img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
