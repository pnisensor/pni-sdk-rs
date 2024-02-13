use targetpoint3::{DataID, TargetPoint3};

fn main() {
    let tp3 = TargetPoint3::connect(None).expect("connects to device");
    let mut tp3 = tp3
        .easy_continuous_mode(0.25, vec![DataID::AccelX])
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

    let mut tp3 = tp3.easy_stop_continuous_mode().unwrap();
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