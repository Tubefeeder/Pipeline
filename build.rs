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

    println!("cargo:rerun-if-changed=po");

    let paths = std::fs::read_dir("./po").unwrap();

    for path in paths {
        let path = path.unwrap();
        if path.file_type().unwrap().is_dir() {
            let path = path.path();
            let pathname = path.display();
            std::process::Command::new("mkdir")
                .arg(format!("{}/LC_MESSAGES", pathname))
                .spawn()
                .unwrap();
            std::process::Command::new("msgfmt")
                .arg(format!("{}/de.schmidhuberj.tubefeeder.po", pathname))
                .arg("-o")
                .arg(format!("{}/LC_MESSAGES/de.schmidhuberj.tubefeeder.mo", pathname))
                .spawn()
                .unwrap();
        }
    }
}
