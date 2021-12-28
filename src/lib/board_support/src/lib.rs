#[cfg(feature = "rollo_esp32_board")]
pub use rollo_esp32_board::*;

#[cfg(feature = "rollo_host_mock_board")]
pub use rollo_host_mock_board::*;

pub mod wifi;

#[cfg(test)]
mod tests {
    #[test]

    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
