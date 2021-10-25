// use safe_transmute::base::transmute_many;
use safe_transmute::guard::AllOrNothingGuard;
use safe_transmute::guard::Guard;
use safe_transmute::guard::PedanticGuard;

struct Zst;

fn main() {
    // let _a = transmute_many::<Zst, SingleManyGuard>(&[0x00, 0x01, 0x00, 0x02]);
    println!("{:?}", AllOrNothingGuard::check::<Zst>(&[0x00, 0x01]));
    println!("{:?}", PedanticGuard::check::<Zst>(&[0x00, 0x01]));
}
