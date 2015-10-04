use earley::UniqVec;
use std::iter::FromIterator;

#[test]
fn test1() {
    let mut uv = UniqVec::new();
    let ins = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 0];
    uv.extend(ins.into_iter());
    assert_eq!(uv.len(), 10);
    uv.push(15);
    assert_eq!(uv.len(), 11);
    uv.push(15);
    assert_eq!(uv.len(), 11);
}

#[test]
fn test2() {
    let ins = vec![1, 1, 2, 2, 1, 3, 4, 3, 5];
    let uv = UniqVec::from_iter(ins);
    assert_eq!(uv.len(), 5);
    assert_eq!(uv[0], 1);
    assert_eq!(uv[1], 2);
    assert_eq!(uv[2], 3);
    assert_eq!(uv[3], 4);
    assert_eq!(uv[4], 5);
}

#[test]
fn test3() {
    let uv = UniqVec::from_iter(vec![1, 1, 2, 2, 1, 3, 4, 3, 5]);
    assert_eq!(uv.len(), 5);
    for (i, n) in uv.iter().enumerate() {
        assert_eq!(uv[i], *n);
    }
}
