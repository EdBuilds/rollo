use std::borrow::BorrowMut;
use bal::wifi::Error;
use bal::wifi::WifiResource;
use board_support::{BoardResourceBuilder, use_wifi};
use board_support::WifiContainer;
fn foo(bar: &mut dyn WifiResource) {
    println!("{:#?}",bar.scan());
}
fn main() {
    let mut board = BoardResourceBuilder::resolve(Default::default()).unwrap();
    foo(use_wifi!(&mut board.wifi, |wf| {wf.borrow_mut()}));
    println!("Hello, world!{:#?}", Error::Undefined);
}
