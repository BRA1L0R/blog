use rocket::fs::FileServer;

#[rocket::launch]
fn rocket() -> _ {
    let env = std::env::var("SERVE_DIR").expect("undefined SERVE_DIR");
    std::env::set_current_dir(&env).unwrap();

    rocket::build()
        .mount("/", FileServer::from("dist/"))
        .mount("/static", FileServer::from("static/").rank(-1))
}
