use remuir::*;

#[test]
fn inc_from_max_test() {
    let mut reg = Register::new(&vec![u128::MAX][..]);
    reg.inc();
    assert_eq!(reg, Register::new(&vec![0, 1]));
}

#[test]
fn inc_units_max_only_test() {
    let mut reg = Register::new(&vec![u128::MAX, u128::MAX, 4, ]);
    reg.inc();
    assert_eq!(reg, Register::new(&vec![0, 0, 5]));
}

#[test]
fn dec_from_0_units() {
    let mut reg = Register::new(&vec![0, 1]);
    reg.dec();
    assert_eq!(reg, Register::new(&vec![u128::MAX]));
}

#[test]
fn is_zero_test() {
    let reg = Register::new(&vec![]);
    let mut mem = Memory::new_from_slice(&[reg][..]);
    assert!(mem.is_zero(&0))
}
