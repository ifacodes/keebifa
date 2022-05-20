use keyberon::action::{k, l, m, Action, Action::*, HoldTapConfig};
use keyberon::key_code::KeyCode::*;
use keyberon::layout::*;

#[rustfmt::skip]
// pub static LAYERS: Layers<13, 5, 2> = layout! {
//     {
//     [        1  2   3  4  5  6      7      8  9  0   -    =   BSpace]
//     ['`'     Q      W  E  R  T      U      I  O  P  '['  ']'    '\\']
//     [Tab     A      S  D  F  G      Y      J  K  L   ;  Quote  Enter]
//     [Escape  LCtrl  Z  X  C  V      n      H  N  ,   .  RShift   (1)]
//     ['`'  n  LShift n  LAlt  Space  LGui   B  Space  M  RAlt ? RCtrl]
//     }
//     {
//         [ t t t t t t t t t t t t t ]
//         [ t t t t t t t t t t t t t ]
//         [ t t t t t t t t t t t t t ]
//         [ t t t t t t t t t t t t t ]
//         [ t t t t t t t t t t t t t ]
//     }
// };

pub static TEST_LAYER: Layers<2, 2, 1> = layout! {
    {
        [     I F],
        [LShift A],
    }
};
