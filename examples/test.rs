use castaway::cast;

fn main() {
    let mut slice = [1u8; 2];

    fn inner2<'a>(value: &'a [u8]) {
        assert_eq!(cast!(value, &[u8]), Ok(&[1, 1][..]));
        assert_eq!(cast!(value, &'a [u8]), Ok(&[1, 1][..]));
        assert_eq!(cast!(value, &'a [u16]), Err(&[1, 1][..]));
        assert_eq!(cast!(value, &'a [i8]), Err(&[1, 1][..]));
    }

    inner2(&slice);
}
