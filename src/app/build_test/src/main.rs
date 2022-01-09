use std::borrow::BorrowMut;
use std::thread;
use std::thread::Thread;
use std::time::Duration;
use bal::wifi::Error;
use bal::wifi::WifiResource;
use board_support::{BoardResourceBuilder, use_wifi};
use board_support::WifiContainer;
use smol;
//#[cfg(target_os = "espidf")]
use esp_idf_sys::link_patches;
use smol::Timer;


fn foo(bar: &mut dyn WifiResource) {
    println!("{:#?}",bar.scan());
}
fn main() {
 //   #[cfg(target_os = "espidf")]
        link_patches();
    println!("Hello!");
    thread::spawn(move  || {
        loop {
            println!("thread1!");
            thread::sleep(Duration::from_secs(1));
        }
    });
    thread::spawn(move  || {
        loop {
            println!("thread2!");
            thread::sleep(Duration::from_secs(1));
        }
    });
}
