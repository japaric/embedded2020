pub fn round_down(x: u16, n: u16) -> u16 {
    let rem = x % n;
    if rem != 0 {
        x - rem
    } else {
        x
    }
}

pub fn round_up(x: u16, n: u16) -> u16 {
    let rem = x % n;
    if rem != 0 {
        n + (x - rem)
    } else {
        x
    }
}
