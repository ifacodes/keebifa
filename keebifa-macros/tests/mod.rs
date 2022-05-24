extern crate keebifa_macros;
use keebifa_macros::alice_layout;
use keyberon::action::Action;

#[cfg(test)]
#[test]
#[rustfmt::skip]
fn test_convert_layer() {
    pub static array: [[Action<core::convert::Infallible>; 65];1] =  alice_layout! {
        {
            [ 1 2 3 4 5 6 7 8 9 0 Q W E R T ]
            [ 1 2 3 4 5 6 7 8 9 0 Q W E R T ]
            [ 1 2 3 4 5 6 7 8 9 0 Q W E R ]
            [ 1 2 3 4 5 6 7 8 9 0 Q W E R ]
            [ 1 2 3 4 5 6 7 ]
        }
    };

    println!("{:?}", array);
}
