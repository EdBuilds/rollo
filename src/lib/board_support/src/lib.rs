use std::marker::PhantomData;
#[cfg(feature = "rollo_esp32_board")]
pub use rollo_esp32_board::*;

#[cfg(feature = "rollo_host_mock_board")]
pub use rollo_host_mock_board::*;

pub mod wifi;

pub struct Board<ST, SW, WI, SE>{
    _phantom_stepper: PhantomData<ST>,
    _phantom_switch: PhantomData<SW>,
    _phantom_wifi: PhantomData<WI>,
    _phantom_server: PhantomData<SE>,
}

#[cfg(test)]
mod tests {
    #[test]

    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
