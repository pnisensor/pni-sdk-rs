use pni_sdk::Device;
use pni_sdk::acquisition::DataID;

fn main() {
    let tp3 = Device::connect(None).expect("connects to device");
    let mut tp3 = tp3
        .continuous_mode_easy(0.25, vec![DataID::AccelX])
        .expect("got into cont mode");
    {
        let mut iter = tp3.iter();
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
    }

    let mut tp3 = tp3.stop_continuous_mode_easy().unwrap();
    {
        let mut iter = tp3.iter();
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
        println!("{:?}", iter.next());
    }
}
