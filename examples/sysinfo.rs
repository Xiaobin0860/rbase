use sysinfo::{Components, Disks, Networks, System};

fn main() {
    // Please note that we use "new_all" to ensure that all list of
    // components, network interfaces, disks and users are already
    // filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    let networks = Networks::new_with_refreshed_list();
    let disks = Disks::new_with_refreshed_list();
    let components = Components::new_with_refreshed_list();

    // We display all disks' information:
    println!("=> disks:");
    for disk in disks.iter() {
        println!("{:?}", disk);
    }

    // Network interfaces name, data received and data transmitted:
    println!("=> networks:");
    for (interface_name, data) in networks.iter() {
        println!("{} {}", interface_name, data.mac_address(),);
    }

    // Components temperature:
    println!("=> components:");
    for component in components.iter() {
        println!("{:?}", component);
    }

    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());

    // Number of CPUs:
    println!("CPUs: {}", sys.physical_core_count().unwrap_or(1));
    for cpu in sys.cpus() {
        println!("{:?}", cpu);
    }

    println!("{:?}", System::load_average());

    println!("{:?}", rbase::get_sys_info());
}
