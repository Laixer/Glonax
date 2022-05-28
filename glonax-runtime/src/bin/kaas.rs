#[tokio::main]
async fn main() {
    let mut y = glonax_gamepad::Gamepad::new(std::path::Path::new("/dev/input/js0"))
        .await
        .unwrap();

    loop {
        let ev = y.next_event().await;
        println!("{:?}", ev);
    }
}
