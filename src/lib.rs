use local_ip_address::list_afinet_netifas;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

const HOSTIDFILE: &str = "/etc/hostid";
const HOSTSFILE: &str = "/etc/hosts";

#[inline]
pub fn gethostid() -> String {
    let fd = File::open(HOSTIDFILE);

    if let Ok(f) = fd {
        let mut reader = BufReader::new(f);

        let mut buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut buf[..]).unwrap();
        buf.reverse();

        let mut s = String::new();
        for byte in buf.bytes() {
            write!(&mut s, "{:x}", byte.unwrap()).expect("Unable to write as hex");
        }

        s
    }
    // Check for host IP (not localhost but hostname associated IP !)
    else {
        let hostname = gethostname().trim().to_string();
        // Open (read-only) hosts net config
        let fd = File::open(HOSTSFILE);

        let mut ip: Option<String> = None;
        if let Ok(f) = fd {
            let mut reader = BufReader::new(f);
            let mut buf = String::new();
            reader.read_to_string(&mut buf).unwrap();

            for spl in buf.split('\n') {
                if spl.contains(&hostname) {
                    ip = Some(spl.split('\t').collect::<Vec<_>>()[0].to_string());
                }
            }
        }

        // else if IP is not found or "hosts" file does not exist we falloff to localhost IP
        let ip = match ip {
            Some(i) => i,
            None => {
                let network_interfaces = list_afinet_netifas().unwrap();
                let (_, i) = local_ip_address::find_ifa(network_interfaces, "lo").unwrap();
                i.to_string()
            }
        };

        // Encode IP to hex with some shifting like in libc : https://codebrowser.dev/glibc/glibc/sysdeps/unix/sysv/linux/gethostid.c.html - L.130
        let mut u_vec = Vec::with_capacity(4);
        for c in ip.split('.') {
            u_vec.push(c.parse().unwrap())
        }
        u_vec.reverse();

        let to_shift = u32::from_le_bytes(u_vec.try_into().unwrap());
        let shifted = to_shift << 16 | to_shift >> 16;

        let mut s = String::new();
        for byte in shifted.to_le_bytes() {
            write!(&mut s, "{:02x}", byte).expect("Unable to write as hex");
        }

        s
    }
}

const ETC_HOSTNAME: &str = "/etc/hostname";
const PROC_HOSTNAME: &str = "/proc/sys/kernel/hostname";

fn gethostname() -> String {
    if let Ok(f) = File::open(ETC_HOSTNAME) {
        _get_host_name(f)
    } else if let Ok(f) = File::open(PROC_HOSTNAME) {
        _get_host_name(f)
    } else {
        panic!("Host does not have hostname");
    }
}

#[inline]
fn _get_host_name(f: File) -> String {
    let mut reader = BufReader::new(f);
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();

    buf
}

//#[cfg(test)]
//mod test {
//    use std::fs::File;
//
//    use crate::gethostid;
//
//    #[test]
//    fn test_host_id() {
//        const HOSTIDFILE: &str = "/etc/hostid";
//
//        let hostid = gethostid();
//        println!("{}", hostid);
//        let fd = File::open(HOSTIDFILE);
//
//        if fd.is_ok() {
//            assert_eq!(hostid, String::from("64616873"))
//        } else {
//            assert_eq!(hostid, String::from("1c7f0100"))
//        }
//    }
//}
