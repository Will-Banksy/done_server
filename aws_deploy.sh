#!/bin/bash

# Creates an EC2 instance and RDS instance in the default VPC with a new security group that allows
# all HTTP/HTTPS/SSH traffic to the EC2 instance, and another two security groups on the RDS and EC2 instances
# to allow traffic between them. The EC2 instance uses a pre-existing SSH key pair, the name of which can
# be configured below under "ec2_key_name"

# Requires:
# - An existing key pair
# - A default VPC
# - AWS credentials stored in ~/.aws/credentials or elsewhere, provided the aws cli is configured to get them from elsewhere by default
# - Sufficient permissions in your AWS account to create EC2 and RDS instances in the default VPC, and to create security groups

ec2_instance_name="sboxws";
ec2_key_name="vockey";
rds_instance_name="sboxdb";
rds_master_user="admin";
rds_master_pass="roottoor";
sg_name="sboxsg"
region="us-east-1";

sgid_ip_permissions='[{"IpProtocol": "tcp","FromPort": 22,"ToPort": 22,"IpRanges": [{"CidrIp": "0.0.0.0/0"}]},{"IpProtocol": "tcp","FromPort": 443,"ToPort": 443,"IpRanges": [{"CidrIp": "0.0.0.0/0"}]},{"IpProtocol": "tcp","FromPort": 80,"ToPort": 80,"IpRanges": [{"CidrIp": "0.0.0.0/0"}]}]';

ec2_rds_sg_name="sboxws-db_sg";

rds_ec2_sg_name="sboxdb-ws_sg";

ec2_block_device_mappings='[{"DeviceName": "/dev/xvda","Ebs": {"Encrypted": false,"DeleteOnTermination": true,"Iops": 3000,"VolumeSize": 30,"VolumeType": "gp3","Throughput": 125}}]';
ec2_tag_specs='[{"ResourceType": "instance","Tags": [{"Key": "Name","Value": '"\"$ec2_instance_name"\"'}]}]';
ec2_meta_opts='{"HttpTokens": "required","HttpEndpoint": "enabled","HttpPutResponseHopLimit": 2}';
ec2_priv_dns_opts='{"HostnameType": "ip-name","EnableResourceNameDnsARecord": true,"EnableResourceNameDnsAAAARecord": false}';

# Create security groups

sgid=$(aws ec2 create-security-group --group-name "$sg_name" --description "Security group for done_server EC2 instance and RDS instance" --region "$region" | awk '/GroupId/ { print $2 }' | tr -d '",');
aws ec2 authorize-security-group-ingress --group-id "$sgid" --ip-permissions "$sgid_ip_permissions" --region "$region";

ec2_rds_sg_ip_permissions='[{"IpProtocol": "tcp","FromPort": 3306,"ToPort": 3306,"IpRanges": [{"CidrIp": "0.0.0.0/0"}]}]';
ec2_rds_sgid=$(aws ec2 create-security-group --group-name "$ec2_rds_sg_name" --description "Security group for done_server on EC2: EC2 to (RDS)" --region "$region" | awk '/GroupId/ { print $2 }' | tr -d '",');
aws ec2 authorize-security-group-egress --group-id "$ec2_rds_sgid" --ip-permissions "$ec2_rds_sg_ip_permissions" --region "$region";

rds_ec2_sg_ip_permissions='[{"IpProtocol": "tcp","FromPort": 3306,"ToPort": 3306,"UserIdGroupPairs": [{"GroupId": '\""$ec2_rds_sgid"\"'}]}]';
rds_ec2_sgid=$(aws ec2 create-security-group --group-name "$rds_ec2_sg_name" --description "Security group for done_server on RDS: (EC2) to RDS" --region "$region" | awk '/GroupId/ { print $2 }' | tr -d '",');
aws ec2 authorize-security-group-ingress --group-id "$rds_ec2_sgid" --ip-permissions "$rds_ec2_sg_ip_permissions" --region "$region";

echo "sgid: $sgid";
echo "ec2_rds_sgid: $ec2_rds_sgid";
echo "rds_ec2_sgid: $rds_ec2_sgid";

# Create EC2 instance

ec2_nics='[{"AssociatePublicIpAddress": true,"DeviceIndex": 0,"Groups": ['"\"$sgid\""', '"\"$ec2_rds_sgid\""']}]';

aws ec2 run-instances --image-id ami-0230bd60aa48260c6 --count 1 --instance-type t2.micro --key-name "$ec2_key_name" --block-device-mappings "$ec2_block_device_mappings" --tag-specifications "$ec2_tag_specs" --metadata-options "$ec2_meta_opts" --private-dns-name-options "$ec2_priv_dns_opts" --network-interfaces "$ec2_nics" --region "$region";

# Create RDS instance

aws rds create-db-instance --db-instance-identifier "$rds_instance_name" --db-instance-class db.t3.micro --engine mariadb --master-username "$rds_master_user" --master-user-password "$rds_master_pass" --vpc-security-group-ids "$sgid" "$rds_ec2_sgid" --allocated-storage 20 --publicly-accessible --backup-retention-period 0 --region "$region";

# Await RDS instance (usually takes longer)

aws rds wait db-instance-available --db-instance-identifier "$rds_instance_name" --region "$region";

echo "Assuming that all ran successfully, there should be an EC2 instance called $ec2_instance_name you can connect to in your AWS account to deploy the server application.";