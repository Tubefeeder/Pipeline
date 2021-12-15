fn main() {
    println!("cargo:rerun-if-changed=resources.gresource.xml");

    std::process::Command::new("glib-compile-resources")
        .arg("resources.gresource.xml")
        .spawn()
        .unwrap();

    println!("cargo:rerun-if-changed=ui");

    std::process::Command::new("glib-compile-resources")
        .arg("resources.gresource.xml")
        .spawn()
        .unwrap();
}
