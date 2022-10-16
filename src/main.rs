
use std::time::Duration;
// use std::String;
use rand::Rng;
use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};

use std::time::SystemTime;

// #[derive(Debug, Copy, Clone)]
#[derive(Debug)]
struct O_button{
    b_down: Option<bool>,
    n_value: Option<u64>,
    s_name: String,
    n_bit: Option<u8>,
    a_n_num: Option<Vec<u8>>,
    n_bit_offset: u32,
    n_bits: u32
}
// #[derive(Debug, Copy, Clone)]
#[derive(Debug)]
struct O_controller{
    s_name: String,
    a_o_button: Vec<O_button>
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn convert_argument(input: &str) -> u16 {
    if input.starts_with("0x") {
        return u16::from_str_radix(input.trim_start_matches("0x"), 16).unwrap();
    }
    u16::from_str_radix(input, 10)
        .expect("Invalid input, be sure to add `0x` for hexadecimal values.")
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("usage: read_device <base-10/0xbase-16> <base-10/0xbase-16>");
        return;
    }

    let vid = convert_argument(args[1].as_ref());
    let pid = convert_argument(args[2].as_ref());

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid) {
            Some((mut device, device_desc, mut handle)) => {
                read_device(&mut device, &device_desc, &mut handle).unwrap()
            }
            None => println!("could not find device {:04x}:{:04x}", vid, pid),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => panic!("Device found but failed to open: {}", e),
            }
        }
    }

    None
}

fn read_device<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Languages: {:?}", languages);

    if !languages.is_empty() {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
    }

    // match find_readable_endpoint(device, device_desc, TransferType::Interrupt) {
    //     Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Interrupt),
    //     None => println!("No readable interrupt endpoint"),
    // }

    // match find_readable_endpoint(device, device_desc, TransferType::Bulk) {
    //     Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Bulk),
    //     None => println!("No readable bulk endpoint"),
    // }


    match f_find_readable_endpoint(
        device,
        device_desc,
        TransferType::Interrupt
    ) {

        Some(endpoint) => f_write_endpoint(handle, endpoint, TransferType::Interrupt),
        None => println!("No readable interrupt endpoint"),
    }
    Ok(())
}
fn f_find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::Out
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn f_write_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
) {

    println!("endpoint {:?}", endpoint);
    println!("Writing to endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };
    let timeout = Duration::from_secs(1);
    println!(" - kernel driver? {}", has_kernel_driver);
    // let mut a_nu8 = [0; 64];

    let mut a_n_u8__input = [
        0x02,0x0c,0x55,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0xfc,0xff,0xff,0xff,0xff,0xff,0xff,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xff,0x00,0x00
    ];
    a_n_u8__input[43] = 0;// player led
    let mut n_time = 0;
    let mut n_i =0;
    let mut n_r = 0;
    let mut n_g = 0;
    let mut n_b = 0;
    let mut rng = rand::thread_rng();

    let mut n_index = 0; 

    let mut n_max = 10000;
    loop{
        n_time += 1;
        // println!("________________________________________");
        n_i = (n_i+1) % 255;
        n_r = (n_i+1) % 255;
        n_g = 255 - (n_i+1) % 255;
        n_b = (n_i+1) % 255;
        a_n_u8__input[43] = n_i;// player led
        // a_n_u8__input[45] = n_r;// touchpad led
        // a_n_u8__input[46] = n_g;// touchpad led
        // a_n_u8__input[47] = n_b;// touchpad led
        // if(n_time % 100 == 0){
        //     n_index+=1;
        //     for n in 0..47{
        //         // a_n_u8__input[n] = rng.gen::<u8>();// left L2 motor led
        //         a_n_u8__input[n_index] = 255;
        //     }
        //     println!(" - write: {:?}", &a_n_u8__input[..47]);




        // }
        let n_wave = ((((n_time % n_max) as f64) * 0.01).sin() * 127.0 + 127.0) as u8;
        // a_n_u8__input[0] = 255;
        for n in 0..(n_wave as f64 /2.0) as u8{
            print!("-");
        }
        println!("");
        // a_n_u8__input[2] = n_i;c
        a_n_u8__input[0] = 0b00000010; 
        a_n_u8__input[1] = 0b11111111; 
        a_n_u8__input[2] = 0b11110111; 
        a_n_u8__input[3] = n_wave; // motor left
        a_n_u8__input[4] = n_wave; // motor right


        if(n_i % 50 > 25){

            a_n_u8__input[22] = 253;// left L2 motor led
            a_n_u8__input[23] = rng.gen::<u8>();// left L2 motor led
            a_n_u8__input[24] = rng.gen::<u8>();// left L2 motor led
            a_n_u8__input[25] = rng.gen::<u8>();// left L2 motor led
            a_n_u8__input[26] = rng.gen::<u8>();// left L2 motor led
            a_n_u8__input[27] = rng.gen::<u8>();// left L2 motor led
            a_n_u8__input[28] = rng.gen::<u8>();// left L2 motor led

    
        }else{
            a_n_u8__input[22] = 0;// left L2 motor led
            a_n_u8__input[23] = 0;// left L2 motor led
            a_n_u8__input[24] = 0;// left L2 motor led
            a_n_u8__input[25] = 0;// left L2 motor led
            a_n_u8__input[26] = 0;// left L2 motor led
            a_n_u8__input[27] = 0;// left L2 motor led
            a_n_u8__input[28] = 0;// left L2 motor led
        }

        
        match handle.write_interrupt(endpoint.address, &mut a_n_u8__input, timeout) {
            Ok(len) => {
                // println!(" - write: {:?}", &a_n_u8__input[..len]);
            }
            Err(err) => {
                // println!("could not write to endpoint: {}", err),
            // 
            }
        }


        
    }

}

fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
) {


    let mut o_controller_dualsense = O_controller{
        s_name: String::from("dualsense"),
        a_o_button: vec![
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("triangle"),
                n_bit: Some(3),
                a_n_num: None,
                n_bit_offset: 8*8+4,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("circle"),
                n_bit: Some(2),
                a_n_num: None,
                n_bit_offset: 8*8+4,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("cross"),
                n_bit: Some(1),
                a_n_num: None,
                n_bit_offset: 8*8+4,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("square"),
                n_bit: Some(0),
                a_n_num: None,
                n_bit_offset: 8*8+4,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("arrow_up"),
                n_bit: None,
                a_n_num: Some(vec![ 0, 7, 1]),
                n_bit_offset: 8*8,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("arrow_right"),
                n_bit: Some(2),
                a_n_num: Some(vec![ 1, 2, 3]),
                n_bit_offset: 8*8,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("arrow_down"),
                n_bit: Some(3),
                a_n_num: Some(vec![ 3, 4, 5]),
                n_bit_offset: 8*8,
                n_bits:4
            },
            O_button{
                b_down: Some(false),
                n_value: None,
                s_name: String::from("arrow_left"),
                n_bit: Some(4),
                a_n_num: Some(vec![ 5, 6, 7]),
                n_bit_offset: 8*8,
                n_bits:4
            },
            O_button{
                b_down: None,
                n_value: Some(0),
                s_name: String::from("L2"),
                n_bit: None,
                a_n_num: None,
                n_bit_offset: 5*8,
                n_bits:8
            },
            O_button{
                b_down: None,
                n_value: Some(0),
                s_name: String::from("R2"),
                n_bit: None,
                a_n_num: None,
                n_bit_offset: 6*8,
                n_bits:8
            }
        ]
    };

    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };
    let timeout = Duration::from_secs(1);
    println!(" - kernel driver? {}", has_kernel_driver);
    let mut a_nu8 = [0; 64];

    let mut a_n_u8__input = [0; 64];
    a_n_u8__input[43] = 0;// player led

    loop{
        println!("________________________________________");

        

        match handle.read_interrupt(endpoint.address, &mut a_nu8, timeout) {
            Ok(len) => {
                let mut n_i = 0;
                let mut n_bits_per_line = 8 * 8;
                let mut s_line = String::from("");
                while(n_i < len){
                    // if()
                    if(n_i % (n_bits_per_line/8) == 0){
                        let mut n_byte = 0;
                        while(n_byte < (n_bits_per_line/8)){
                            print!(" {:08b} ", &a_nu8[n_i+n_byte]);
                            n_byte+=1;
                        }
                        println!(" {:?}", SystemTime::now());
                        // println!("{}",s_line);
                        // s_line = String::from("");
                        // println!("")
                    }else{
                        // s_line.push_str(&String::from(format!("{:#08b}", &buf[n_i])))
                        // s_line.push_str(&String::from("asdf"));
                    }
                    // print!("flags: {:#08b}", &buf[n_i]);
                    n_i+=1;
                }
                // println!("flags: {:#018b}", flags);
                // println!(" - read: {:?}", &buf[..len]);
            }
            Err(err) => println!("could not read from endpoint: {}", err),
        }
        print!("{}[2J", 27 as char);


        // match handle.read_interrupt(
        //     endpoint.address,
        //     &mut a_nu8,
        //     timeout
        // ){
        //     Ok(len)=>{
        //         let mut n_index_a_o_button = 0;
        //         while(n_index_a_o_button < o_controller_dualsense.a_o_button.len()){
        //             let o_button = &mut o_controller_dualsense.a_o_button[n_index_a_o_button];

        //             let n_byte_index = ((o_button.n_bit_offset) as f64 / 8 as f64) as u32;
        //             let n_bits_right_shift = o_button.n_bit_offset % 8;
        //             let n = a_nu8[n_byte_index as usize] >> n_bits_right_shift & (((2 as i32).pow(o_button.n_bits)-1) as u8);

        //             if(o_button.n_bit != None){
        //                 o_button.b_down = Some((n & (((2 as i32).pow(o_button.n_bit.unwrap().into())) as u8)) != 0);
        //             }
        //             if(o_button.a_n_num != None){
        //                 o_button.b_down = Some(o_button.a_n_num.as_ref().unwrap().contains(&n));
        //             }
        //             if(o_button.n_value != None){
        //                 o_button.n_value = Some(n.into());
        //             }
        //             // println!("{}:{}", o_button.s_name, n);
        //             println!("{:?}", o_button);

        //             n_index_a_o_button+=1;
        //         }

        //         // let n_u8_triangle_circle_cross_square = a_nu8[8] >> 4;
        //         // println!("n_u8_triangle_circle_cross_square: {:08b}", n_u8_triangle_circle_cross_square);

        //         // let n_u8_arrow_buttons = a_nu8[8] & 0b00001111;
        //         // println!("n_u8_arrow_buttons: {:08b}", n_u8_arrow_buttons);




        //     }
        //     Err(o_error) => {

        //         println!("could not read from endpoint! {}", o_error);
        //     }
        // }

    }

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut buf = [0; 256];
            let timeout = Duration::from_secs(1);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, &mut buf, timeout) {
                        Ok(len) => {
                            println!(" - read: {:?}", &buf[..len]);
                        }
                        Err(err) => println!("could not read from endpoint: {}", err),
                    }
                }
                TransferType::Bulk => match handle.read_bulk(endpoint.address, &mut buf, timeout) {
                    Ok(len) => {
                        println!(" - read: {:?}", &buf[..len]);
                    }
                    Err(err) => println!("could not read from endpoint: {}", err),
                },
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

// fn main(){
//     let s = String::from("asdf");
// }