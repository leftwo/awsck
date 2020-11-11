extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate tokio;

use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};

#[tokio::main]
async fn main() {
    let ec2_client = Ec2Client::new(Region::UsEast2);
    let describe_instances_request = DescribeInstancesRequest::default();

    let all = match ec2_client
        .describe_instances(describe_instances_request)
        .await
    {
        Ok(output) => output,
        Err(error) => {
            println!("Bad {:?}", error);
            return;
        }
    };

    // println!("instance {:?}", all_instances);
    for reservations in all.reservations {
        for res in reservations {
            let inst = match res.instances {
                Some(x) => x,
                None => continue,
            };

            // Each instance is a vector.  And for me there is never
            // more than one item in that vector.
            for (_, x) in inst.iter().enumerate() {
                println!("id {:#?}", x.instance_id.as_ref().unwrap());
                println!("time {:#?}", x.launch_time);
                println!("type {:#?}", x.instance_type);
                println!("key_name {:#?}", x.key_name);
                println!("vpc {:#?}", x.vpc_id);
                println!("subnet {:#?}", x.subnet_id);
                if x.state.is_some() {
                    let state = x.state.as_ref().unwrap();
                    println!("state {:#?}", state.name);
                }
                for (_, y) in x.security_groups.iter().enumerate() {
                    for (_, z) in y.iter().enumerate() {
                        println!("sg_id {:#?}", z.group_id);
                        println!("sg_id {:#?}", z.group_name);
                    }
                }
                println!("pri_ip {:#?}", x.private_ip_address);
                println!("pub_ip {:#?}", x.public_ip_address);
                if  x.placement.is_some() {
                    let plac = x.placement.as_ref().unwrap();
                    println!("az {:#?}", plac.availability_zone);
                }
                println!("tags {:#?}", x.tags);
                if x.tags.is_some() {
                    for (_, y) in x.tags.iter().enumerate() {
                        for (_, z) in y.iter().enumerate() {
                            println!("tags.name {:#?}", z.key);
                            println!("tags.value {:#?}", z.value);
                        }
                    }
                }
            }
        }
    }
}
