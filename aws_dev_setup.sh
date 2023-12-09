#!/bin/bash

# Sets up a minimal Amazon RDS instance for dev work (only accessible from the host's public IP address). Requires the aws cli, and credentials stored in ~/.aws/credentials
# To connect, go into the AWS management console, navigate to the RDS instance, and you should find the connection info there

db_instance_name="sboxdb";
master_user="admin";
master_pass="roottoor";
region="us-east-1";

aws rds create-db-instance --db-instance-identifier $db_instance_name --db-instance-class db.t3.micro --engine mariadb --master-username $master_user --master-user-password $master_pass --allocated-storage 20 --publicly-accessible --backup-retention-period 0 --region $region;

sgid=$(aws rds describe-db-instances --db-instance-identifier sboxdb --region $region | awk '/VpcSecurityGroupId/ { print $2 }' | tr -d '",');

sgrid=$(aws ec2 describe-security-group-rules --filters Name="group-id",Values="$sgid" --region $region | awk '/SecurityGroupRuleId/ { sgrid = $2 } /"IsEgress": false/ { print sgrid }' | tr -d '",');

pub_ip=$(curl ipinfo.io/ip);

aws ec2 revoke-security-group-ingress --group-id "$sgid" --security-group-rule-ids "$sgrid" --region $region;

aws ec2 authorize-security-group-ingress --group-id "$sgid" --ip-permissions "IpProtocol"="-1","FromPort"=-1,"ToPort"=-1,"IpRanges"="[{CidrIp=${pub_ip}/32}]" --region $region;

aws rds wait db-instance-available --db-instance-identifier sboxdb --region $region;

echo "Amazon RDS database instance is now available!";