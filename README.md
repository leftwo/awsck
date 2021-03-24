# awsck  AWS check tool

A simple program that lists AWS EC2 instances.
The default output includes the Name tag, the system state, if the
instance is a Spot instance, the launch date, the EC2 type, the
public I.P. address, and the Availability zone.

For this program to work, you must have your .aws directory configured
with the config and credentials populated.

Add the `-a` flag to also show hosts recently terminated, the VPC,
and detailed launch time info.

Sample output:
```
alan$ awsck
              Name     Launched        Type       Public IP  Avil-zone
Alan Build new nbe R   09/04/20   t3a.large   54.90.149.190 us-east-1b
  newms-DEEP-0-0-0 R   06/11/20    t3.small      3.82.94.54 us-east-1a
 newapp-EDGE-0-1-0 R   06/22/20    t3.large    3.94.166.165 us-east-1b
        NewJenkins R   09/04/20 m5a.2xlarge   18.217.27.177 us-east-2a
              saml R   12/01/20    t3.small  18.216.220.150 us-east-2a
     Alan ohiokube R   04/28/20   r5.xlarge   52.15.193.117 us-east-2a
               r8b R   01/09/21   t3.xlarge   18.218.13.111 us-east-2a
       Standalone2 R S 01/08/21    t3.large   3.140.182.194 us-east-2a
             alan3 R   12/31/20  t3a.xlarge  18.236.101.231 us-west-2b
   recv-EDGE-0-1-0 R S 01/22/21   t3.xlarge  34.222.142.227 us-west-2b
   recv-EDGE-0-0-0 R S 01/22/21   t3.xlarge   54.149.166.86 us-west-2a
```
