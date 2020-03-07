pub fn zigzag(x: i32) -> u32 {
    ((x << 1) ^ (x >> 31)) as u32
}

#[cfg(test)]
mod tests {
    #[test]
    fn zigzag() {
        assert_eq!(super::zigzag(0), 0);
        assert_eq!(super::zigzag(-1), 0b001);
        assert_eq!(super::zigzag(1), 0b010);
        assert_eq!(super::zigzag(-2), 0b011);
        assert_eq!(super::zigzag(2), 0b100);
        assert_eq!(super::zigzag(i32::min_value()), 0xffffffff);
        assert_eq!(super::zigzag(i32::max_value()), 0xfffffffe);
    }
}
