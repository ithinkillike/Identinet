use u24::u24;

//This is just to test out u24, dunno why it wont work.
fn main() {
    let value = u24::from_bytes([1, 2, 3]);
    println!("Success! 24-bit value: {:?}", value);
}
//Data types to be able to convert data between them.
// pub enum DataTypeSelection {
    // OneByte(u8),
    // ThreeByte(u24)
// }