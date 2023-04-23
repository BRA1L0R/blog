fn main() {
    println!("cargo:rerun-if-changed=src/template.html");
    println!("cargo:rerun-if-changed=src/style.css")
}
