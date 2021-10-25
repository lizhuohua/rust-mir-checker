// This would cause an integer overflow
// https://github.com/magiclen/qrcode-generator/issues/2

use qrcode_generator::to_image_from_str;
use qrcode_generator::QrCodeEcc;

fn main() {
    to_image_from_str("hello", QrCodeEcc::Low, std::usize::MAX);
}
