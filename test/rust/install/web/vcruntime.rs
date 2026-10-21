use super::*;

#[test]
fn cab_headers() {
    let mut bytes = vec![0u8; 10];
    let mut cab = b"MSCF\0\0\0\0".to_vec();
    cab.extend_from_slice(&100u32.to_le_bytes());
    cab.resize(100, 0);
    cab[24] = 3;
    cab[25] = 1;
    bytes.extend_from_slice(&cab);
    bytes.extend_from_slice(b"MSCF\0\0\0\0junk");
    assert_eq!(find_cabs(&bytes), vec![(10, 100)]);
}
