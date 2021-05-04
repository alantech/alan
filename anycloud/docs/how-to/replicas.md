# Define minimum and maximum number of VMs per region or cloud

While defining the configuration, Anycloud will ask for a minimum number of VMs to keep running on each region/cloud defined. The same will happen for a maximum if you would like to specify one. Anycloud autoscaling will take these limits into account. By default, the minimum number of VMs is one and no maximum defined, meaning scaling as much as needed.

## Defining a minimum

```bash
$ anycloud config add
Name for new Deploy Config: production
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Region name › us-west-1
? Virtual Machine Type › t2.medium
? Do you want to add another region to this Deploy Config? (n) › n
? Minimum number of VMs per region or cloud (1) › 2
? Would you like to define a maximum number of VMs? (n) › n
Successfully created "production" Credentials
```

## Defining a maximum

```bash
$ anycloud config add
Name for new Deploy Config: production
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Region name › us-west-1
? Virtual Machine Type › t2.medium
? Do you want to add another region to this Deploy Config? (n) › n
? Minimum number of VMs per region or cloud (1) ›
? Would you like to define a maximum number of VMs? (n) › y
Maximum number of VMs per region or cloud 5
Successfully created "production" Credentials
```
