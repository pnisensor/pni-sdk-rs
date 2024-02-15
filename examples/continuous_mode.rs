use pni_sdk::acquisition::DataID;
use pni_sdk::Device;

fn main() {
    let tp3 = Device::connect(None).expect("connects to device");
    let mut tp3 = tp3
        .continuous_mode_easy(0.25, vec![DataID::AccelX])
        .expect("got into cont mode");
    for data in tp3.iter() {
        println!("{:?}", data);
    }
}
