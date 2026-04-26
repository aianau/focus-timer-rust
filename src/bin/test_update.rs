fn main() {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("aianau")
        .repo_name("focus-timer-rust")
        .bin_name("focus-timer-rust.exe")
        .target("windows.zip")
        .current_version("0.1.0")
        .bin_install_path("test_install.exe")
        .build().unwrap();
    match updater.update() {
        Ok(s) => println!("Success: {}", s.version()),
        Err(e) => println!("Error: {}", e),
    }
}