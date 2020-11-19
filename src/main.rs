extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate tokio;

use chrono::{DateTime, NaiveDateTime};
use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};

#[derive(Debug)]
pub struct Instance {
    instance_id: String,
    name: String,
    az: String,
    pub_ip: String,
    inst_type: String,
    pri_ip: String,
    subnet: String,
    key_name: String,
    state: String,
    launch_time_sec: i64,
    vpc: String,
    isl: String,
    spot: String,
}

/// Take a valid EC2 instance from rusoto and use the values
/// in it to fill up a new Instance struct.  Once it's filled,
/// add the new Instance to the provided vector of all instances.
///
/// Return the length of the name we found.
fn instance_to_struct(x: &rusoto_ec2::Instance, all_instances: &mut Vec<Instance>) -> usize {
    let mut name_max = 0;
    let mut inst_name: String = "".to_string();
    for (_, y) in x.tags.iter().enumerate() {
        for (_, z) in y.iter().enumerate() {
            let n = match &z.key {
                Some(p) => p,
                None => continue,
            };
            if n == "Name" {
                inst_name = z.value.clone().unwrap_or_default();
                name_max = inst_name.len();
            }
        }
    }

    let mut inst_az: String = "".to_string();
    if x.placement.is_some() {
        inst_az = x
            .placement
            .as_ref()
            .unwrap()
            .availability_zone
            .clone()
            .unwrap_or_default();
    }

    let mut spot = " ".to_string();
    if x.instance_lifecycle.is_some() && x.instance_lifecycle.clone().unwrap() == "spot" {
        spot = "S".to_string();
    }
    let mut inst_state: String = "?".to_string();
    let mut inst_state_letter: String = "?".to_string();
    if let Some(state) = &x.state {
        inst_state = state.name.clone().unwrap_or_default();
        if let Some(code) = &state.code {
            inst_state_letter = match code {
                0 => 'P'.to_string(),
                16 => 'R'.to_string(),
                32 => 'D'.to_string(),
                48 => 'T'.to_string(),
                64 => 's'.to_string(),
                80 => 'S'.to_string(),
                _ => '?'.to_string(),
            }
        }
    }

    // Get the time string from AWS, reformat it to seconds
    // I'm using seconds here, because it's easier to sort by
    // an i64 than a string.
    let time = x.launch_time.clone().unwrap_or_default();
    let lt_seconds = match DateTime::parse_from_rfc3339(&time) {
        Ok(t) => t.timestamp(),
        Err(e) => {
            println!(
                "Error finding start time for instance {}:\n{}",
                inst_name, e
            );
            0
        }
    };

    let ai: Instance = Instance {
        name: inst_name,
        instance_id: x.instance_id.clone().unwrap_or_default(),
        az: inst_az,
        pub_ip: x.public_ip_address.clone().unwrap_or_default(),
        inst_type: x.instance_type.clone().unwrap_or_default(),
        pri_ip: x.private_ip_address.clone().unwrap_or_default(),
        subnet: x.subnet_id.clone().unwrap_or_default(),
        key_name: x.key_name.clone().unwrap_or_default(),
        state: inst_state,
        launch_time_sec: lt_seconds,
        vpc: x.vpc_id.clone().unwrap_or_default(),
        isl: inst_state_letter,
        spot,
    };
    all_instances.push(ai);
    name_max
}

fn display_instances(all_instances: &mut Vec<Instance>, max: usize) {
    // Header first
    print!("{0:>1$}", "Name".to_string(), max);
    print!(" {0:1}", " ".to_string());
    print!(" {0:1}", " ".to_string());
    print!(" {0:>13}", "Launch Time".to_string());
    print!(" {0:>11}", "Type".to_string());
    print!(" {0:>15}", "Public IP".to_string());
    print!(" {0:>10}", "Avil-zone".to_string());
    // Putting VPC at 12 lines up with the smaller VPC ID length.
    println!(" {0:>12}", "VPC".to_string());

    for y in all_instances.iter() {
        print!("{0:>1$}", y.name, max);
        print!(" {0:1}", y.isl);
        print!(" {0:1}", y.spot);
        let lt_str = NaiveDateTime::from_timestamp(y.launch_time_sec, 0);
        print!(" {0:>13}", lt_str.format("%y/%m/%d %H:%M").to_string());
        print!(" {0:>11}", y.inst_type);
        print!(" {0:>15}", y.pub_ip);
        print!(" {0:>10}", y.az);
        print!(" {0:<21}", y.vpc);
        println!();
    }
}

async fn find_instances(region: Region, all_instances: &mut Vec<Instance>) -> usize {
    let ec2_client = Ec2Client::new(region);
    let describe_instances_request = DescribeInstancesRequest::default();

    let all = match ec2_client
        .describe_instances(describe_instances_request)
        .await
    {
        Ok(output) => output,
        Err(error) => {
            println!("Bad {:?}", error);
            return 0;
        }
    };

    let mut name_max = 0;
    if let Some(reservations) = all.reservations {
        for res in reservations {
            let inst = match res.instances {
                Some(x) => x,
                None => continue,
            };

            // Each instance is a vector.  And for me there is never
            // more than one item in that vector.  Not sure if that is
            // always true, or just true in the way I use AWS.
            for (_, x) in inst.iter().enumerate() {
                let max = instance_to_struct(x, all_instances);
                if max > name_max {
                    name_max = max
                }
            }
        }
    }
    name_max
}

#[tokio::main]
async fn main() {
    let mut all_instances: Vec<Instance> = Vec::with_capacity(4);
    let regions: [Region; 4] = [Region::UsEast1, Region::UsEast2, Region::UsWest1, Region::UsWest2];
    let mut max: usize = 0;

    for r in regions.iter() {
        let name_len: usize = find_instances(r.clone(), &mut all_instances).await;
        if name_len > max {
            max = name_len;
        }
    }
    display_instances(&mut all_instances, max);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        // Test of struct filled with sane values.
        // We don't actually test all of these, but some I'm keeping
        // around in the event I decide to.

        // TODO: add network interface struct
        let mut all_instances: Vec<Instance> = Vec::with_capacity(1);

        let mut p: rusoto_ec2::Placement = rusoto_ec2::Placement::default();
        p.availability_zone = Some("us-east-2a".to_string());
        p.group_name = Some("".to_string());
        p.tenancy = Some("default".to_string());

        let mut test_x: rusoto_ec2::Instance = rusoto_ec2::Instance::default();
        test_x.ami_launch_index = Some(0);
        test_x.client_token = Some("".to_string());
        test_x.ebs_optimized = Some(true);
        test_x.image_id = Some("ami-0c03383ab60707ed9".to_string());
        test_x.instance_id = Some("i-0c6fc50204fc67f25".to_string());
        test_x.instance_type = Some("t3.small".to_string());
        test_x.key_name = Some("SSHKeyName".to_string());
        test_x.launch_time = Some("2038-01-19T03:07:08.000Z".to_string());
        test_x.placement = Some(p);
        test_x.private_dns_name = Some("ip-172-31-11-214.us-east-2.compute.internal".to_string());
        test_x.private_ip_address = Some("127.0.0.1".to_string());
        test_x.public_dns_name = Some("".to_string());
        test_x.root_device_name = Some("/dev/dssd".to_string());
        test_x.root_device_type = Some("ebs".to_string());
        test_x.source_dest_check = Some(true);
        test_x.state = Some(rusoto_ec2::InstanceState {
            code: Some(80),
            name: Some("stopped".to_string()),
        });
        test_x.state_transition_reason =
            Some("User initiated (1971-07-15 10:47:26 GMT)".to_string());
        test_x.subnet_id = Some("subnet-923ffbfb".to_string());
        test_x.tags = Some(vec![rusoto_ec2::Tag {
            key: Some("Name".to_string()),
            value: Some("longlength".to_string()),
        }]);
        test_x.virtualization_type = Some("hvm".to_string());
        test_x.vpc_id = Some("vpc-e6eb188f".to_string());

        let max = instance_to_struct(&test_x, &mut all_instances);
        assert_eq!(max, 10);
        assert_eq!(1, all_instances.len());

        let max = instance_to_struct(&test_x, &mut all_instances);
        assert_eq!(max, 10);

        assert_eq!(2, all_instances.len());
        for y in all_instances.iter() {
            assert_eq!("longlength".to_string(), y.name);
            assert_eq!("S".to_string(), y.isl);
            assert_eq!(2147483228, y.launch_time_sec);
            assert_eq!("t3.small".to_string(), y.inst_type);
            assert_eq!("us-east-2a".to_string(), y.az);
        }
    }

    #[test]
    fn empty() {
        let mut all_instances: Vec<Instance> = Vec::with_capacity(3);
        let test_empty: rusoto_ec2::Instance = rusoto_ec2::Instance::default();
        let max = instance_to_struct(&test_empty, &mut all_instances);
        assert_eq!(max, 0);
        assert_eq!(1, all_instances.len());
    }
    #[test]
    fn placement() {
        let mut all_instances: Vec<Instance> = Vec::with_capacity(3);
        let p: rusoto_ec2::Placement = rusoto_ec2::Placement::default();
        let mut test_placement: rusoto_ec2::Instance = rusoto_ec2::Instance::default();
        test_placement.placement = Some(p);
        let max = instance_to_struct(&test_placement, &mut all_instances);
        assert_eq!(max, 0);
        assert_eq!(1, all_instances.len());
    }
    #[test]
    fn name() {
        let mut all_instances: Vec<Instance> = Vec::with_capacity(3);
        let mut test_name: rusoto_ec2::Instance = rusoto_ec2::Instance::default();
        test_name.tags = Some(vec![rusoto_ec2::Tag {
            key: Some("Name".to_string()),
            value: Some("name".to_string()),
        }]);

        let max = instance_to_struct(&test_name, &mut all_instances);
        assert_eq!(max, 4);
        assert_eq!(1, all_instances.len());
        for y in all_instances.iter() {
            assert_eq!("name".to_string(), y.name);
        }
    }
}
