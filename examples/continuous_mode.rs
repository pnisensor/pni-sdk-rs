use targetpoint3::acquisition::DataID;
use targetpoint3::TargetPoint3;

fn main() {
    let tp3 = TargetPoint3::connect(None).expect("connects to device");
    let mut tp3 = tp3
        .continuous_mode_easy(0.25, vec![DataID::AccelX])
        .expect("got into cont mode");
    for data in tp3.iter() {
        println!("{:?}", data);
    }
}
