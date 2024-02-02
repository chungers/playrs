#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use std::convert::TryInto;

#[test]
fn test_build_edge_key() {
    // head, tail ids
    let k = (1u64, 2u64);

    let b = (k.0.to_le_bytes(), k.1.to_le_bytes());

    // combine [u64; 8] into Vec[u8]
    let mut kk: Vec<u8> = b.0.to_vec();
    kk.extend(b.1.to_vec());
    println!("first, second = {:?}", kk);

    let buff: &Vec<u8> = &kk;
    println!("buff = {:?}", buff);

    assert_eq!(kk, *buff);

    // make sure number of octets line up
    assert_eq!(std::mem::size_of::<u64>() * 2, buff.len());

    let parts = (
        &buff[0..std::mem::size_of::<u64>()],
        &buff[std::mem::size_of::<u64>()..buff.len()],
    );

    // compare the slices; useful for terminating prefix scans of edges
    assert!(b.0 == parts.0);

    let parsed = (
        u64::from_le_bytes(parts.0.try_into().unwrap()),
        u64::from_le_bytes(parts.1.try_into().unwrap()),
    );
    println!("read keys {:?}, {:?}", parsed.0, parsed.1);

    assert_eq!(k, parsed);
}
