fn main() {
    let (a, b, c, d) = (0, 0, 0, 0);
    let _ = u32::from_be(((a as u32) << 24) | 
                         ((b as u32) << 16) | 
                         ((c as u32) << 8) | 
                         (d as u32) << 0);
}
