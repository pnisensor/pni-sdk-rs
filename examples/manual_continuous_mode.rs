use targetpoint3::AcqParams;
use targetpoint3::DataID;
use targetpoint3::TargetPoint3;

fn main() {
    let mut tp3 = TargetPoint3::connect(None).expect("Connect to TP3");
    println!("Module Info: \n{:?}", tp3.get_mod_info());
    println!("Serial Number: \n{:?}", tp3.serial_number());
    println!(
        "SetAcqParams: \n{:?}",
        tp3.set_acq_params(AcqParams {
            acquisition_mode: false,
            flush_filter: false,
            sample_delay: 0.01
        })
    );
    println!(
        "Set Data Components: \n{:?}",
        tp3.set_data_components(vec![DataID::AccelX])
    );
    println!("Get Data Components: \n{:?}", tp3.get_data());
    println!("Set Cont Mode: \n{:?}", tp3.start_continuous_mode());
    println!("Save config: \n{:?}", tp3.save());
    println!("Power down: \n{:?}", tp3.power_down());

    tp3 = TargetPoint3::connect(None).expect("Connect to TP3");
    println!("Power up result {:?}", tp3.power_up());
    println!("S/N result {:?}", tp3.serial_number());
    for data in tp3.iter() {
        println!("{:?}", data)
    }
}
