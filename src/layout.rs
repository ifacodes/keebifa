use keebifa_macros::alice_layout;
use keyberon::action::Action::{self, *};
use keyberon::layout::*;

const fn arrange_layer(input: [Action; 65]) -> [[Action; 13]; 5] {
    [
        [
            input[2], input[3], input[4], input[5], input[6], input[7], input[8], input[9],
            input[10], input[11], input[12], input[13], input[14],
        ],
        [
            input[1], input[17], input[18], input[19], input[20], input[21], input[23], input[24],
            input[25], input[26], input[27], input[28], input[29],
        ],
        [
            input[16], input[32], input[33], input[34], input[35], input[36], input[22], input[38],
            input[39], input[40], input[41], input[42], input[43],
        ],
        [
            input[0], input[31], input[45], input[46], input[47], input[48], input[49], input[37],
            input[51], input[53], input[54], input[56], input[57],
        ],
        [
            input[15], input[30], input[44], input[58], input[59], input[60], input[61], input[50],
            input[62], input[52], input[63], input[55], input[64],
        ],
    ]
}

const fn convert_layers<const L: usize>(
    input: [[Action<core::convert::Infallible>; 65]; L],
) -> Layers<13, 5, L> {
    let i = 0;
    let mut new_layers: [[[Action<core::convert::Infallible>; 13]; 5]; L] =
        [[[Action::NoOp; 13]; 5]; L];
    while i <= L {
        new_layers[i] = arrange_layer(input[i]);
    }
    new_layers
}

#[rustfmt::skip]
#[allow(dead_code)]

pub static ALICE_LAYOUT: Layers<13, 5, 1> = convert_layers(alice_layout! {
    {
        [Escape '`' 1 2 3 4 5 6 7 8 9 0 - = BSpace]
        [PgUp Tab Q W E R T Y U I O P '[' ']' '\\']
        [PgDown LCtrl A S D F G H J K L ; Quote Enter]
        [LShift Z X C V n B N M , . / RShift n]
        [n LAlt Space LGui Space RAlt RCtrl]
    }
});

#[cfg(test)]
#[test]
fn alice_layout_test() {
    pub static ALICE_LAYOUT: Layers<13, 5, 1> = convert_layers(alice_layout! {
        {
            [Escape '`' 1 2 3 4 5 6 7 8 9 0 - = BSpace]
            [PgUp Tab Q W E R T Y U I O P '[' ']' '\\']
            [PgDown LCtrl A S D F G H J K L ; Quote Enter]
            [LShift Z X C V n B N M , . / RShift n]
            [n LAlt Space LGui Space RAlt RCtrl]
        }
    });
    printf!("{:?}", ALICE_LAYOUT);
}
