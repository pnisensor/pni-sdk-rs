use targetpoint3::TargetPoint3;

fn main() {
    let mut tp3 = TargetPoint3::connect(None).expect("connects to device");
    println!(
        "Module Info: {}",
        tp3.get_mod_info().expect("Couldn't get module info")
    );
    println!(
        "Serial Number: {}",
        tp3.serial_number().expect("Couldn't get serial number")
    );
}
