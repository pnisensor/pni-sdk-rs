use pni_sdk::Device;

fn main() {
    let mut tp3 = Device::connect(None).expect("connects to device");
    println!(
        "Module Info: {}",
        tp3.get_mod_info().expect("Couldn't get module info")
    );
    println!(
        "Serial Number: {}",
        tp3.serial_number().expect("Couldn't get serial number")
    );
}
