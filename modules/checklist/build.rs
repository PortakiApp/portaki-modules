fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=i18n/");
    println!("cargo:rerun-if-changed=email_i18n/");
}
