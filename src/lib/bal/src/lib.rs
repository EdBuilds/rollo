#![feature(trait_alias)]
pub mod wifi;
pub mod switch;
pub mod stepper;
pub mod server;
pub mod networking_types;
pub mod client;

pub trait Takeable {
   fn take<R>(&mut self, id: usize) -> Option<R>;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
