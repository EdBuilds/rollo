#![feature(trait_alias)]
pub mod wifi;
pub mod switch;
pub mod stepper;
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
