extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate tokio;

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
    launch_time: String,
    vpc: String,
    isl: String,
    spot: String,
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

    let mut name_len = 0;
    // println!("{:#?}", all.reservations);
    if let Some(reservations) = all.reservations {
        for res in reservations {
            let inst = match res.instances {
                Some(x) => x,
                None => continue,
            };

            // Each instance is a vector.  And for me there is never
            // more than one item in that vector.
            for (_, x) in inst.iter().enumerate() {
                let mut inst_name: String = "".to_string();
                for (_, y) in x.tags.iter().enumerate() {
                    for (_, z) in y.iter().enumerate() {
                        let n = match &z.key {
                            Some(p) => p,
                            None => continue
                        };
                        if n == "Name" {
                            inst_name = z.value.clone().unwrap_or_default();
                            let len = inst_name.len();
                            if len > name_len {
                                name_len = len;
                            }
                        }
                    }
                }
                let mut inst_az: String = "".to_string();
                if  x.placement.is_some() {
                    inst_az = x.placement.as_ref().unwrap().availability_zone.clone().unwrap_or_default();
                }

                let mut spot = " ".to_string();
                if x.instance_lifecycle.is_some() &&
                   x.instance_lifecycle.clone().unwrap() == "spot" {
                    println!("spotttt");
                    spot = "S".to_string();
                }
                let mut inst_state: String = "?".to_string();
                let mut inst_state_letter: String = "X".to_string();
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
                    launch_time: x.launch_time.clone().unwrap_or_default(),
                    vpc: x.vpc_id.clone().unwrap_or_default(),
                    isl: inst_state_letter,
                    spot: spot,
                };
                all_instances.push(ai);
            }
        }
    }
    return name_len
}

#[tokio::main]
async fn main() {
    let mut all_instances: Vec<Instance> = Vec::with_capacity(4);

    let mut max = 0;
    // This will be redone as a loop, I promise!
    /*
    let name_len: usize = find_instances(Region::UsEast1, &mut all_instances).await;
    if name_len > max { max = name_len };
    */
    let name_len: usize = find_instances(Region::UsEast2, &mut all_instances).await;
    if name_len > max { max = name_len };
    /*
    let name_len: usize = find_instances(Region::UsWest1, &mut all_instances).await;
    if name_len > max { max = name_len };
    let name_len: usize = find_instances(Region::UsWest2, &mut all_instances).await;
    if name_len > max { max = name_len };
    */
    while let Some(y) = all_instances.pop() {
        print!("{0:>1$}", y.name, max);
        print!(" {0:1}", y.isl);
        print!(" {0:1}", y.spot);
        print!(" {}", y.launch_time);
        print!(" {0:>11}", y.inst_type);
        print!(" {0:>15}", y.pub_ip);
        print!(" {0:>10}", y.az);
        println!();
    }
}