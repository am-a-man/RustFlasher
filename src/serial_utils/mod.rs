use serial_enumerator::{get_serial_list, SerialInfo};

#[derive(Debug)]
pub struct SerialItem {
    name: String,
    vendor: String,
    product: String,
    usb: String,
}

impl SerialItem {
    pub fn from_serial_info(serial_info: SerialInfo) -> SerialItem {

        let field_or_else = || Some(String::from("--"));
        SerialItem {
            name: serial_info.name,
            vendor: serial_info.vendor.or_else(field_or_else).unwrap(),
            product: serial_info.product.or_else(field_or_else).unwrap(),
            usb: serial_info
                .usb_info
                .and_then(|usbinfo| Some(format!("{}:{}", usbinfo.vid, usbinfo.pid)))
                .or_else(field_or_else)
                .unwrap(),
        }
    }
    
    pub fn serial_list() -> Vec<SerialItem> {
        let serials_info = get_serial_list();
        let mut serials_table = Vec::new();
        
        for serial_info in serials_info {
            serials_table.push(SerialItem::from_serial_info(serial_info));
        }

        serials_table
    }

    pub fn get_name(&self) -> &String {
        &(self.name)
    }

    pub fn get_vendor(&self) -> &String {
        &self.vendor
    }

    pub fn get_product(&self) -> &String {
        &self.product
    }

    pub fn get_usb(&self) ->    &String {
        &self.usb
    }
}
