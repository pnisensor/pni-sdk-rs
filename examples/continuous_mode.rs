use targetpoint3::{DataID, TargetPoint3};

fn main() {
    let tp3 = TargetPoint3::connect(None).expect("connects to device");
    let mut tp3 = tp3
        .easy_continuous_mode(0.25, vec![DataID::AccelX])
        .expect("got into cont mode");
    for data in tp3.iter() {
        println!("{:?}", data);
    }
}
