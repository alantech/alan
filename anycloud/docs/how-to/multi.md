# Deploy an app to multiple clouds and/or regions

AnyCloud makes it trivial to deploy a singular logical cluster, or application, to multiple regions and/or multiple clouds at the same time. AnyCloud will always keep at least one running server in each region/cloud defined.

## Multiple regions

Generate a Deploy Config across AWS regions `us-west-1` and `us-west-2`.

```bash
$ anycloud config new
Name for new Deploy Config: production
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › us-west-1
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › t2.medium
? Do you want to add another region to this Deploy Config? y
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › us-west-2
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › t2.medium
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "production" Deploy Config.
```

## Multiple clouds

Generate a Deploy Config across AWS region `us-west-1` and GCP region `us-west1-c`.

```bash
$ anycloud config new
? Name for new Deploy Config (staging) › production
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › us-west-1
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › t2.medium
? Do you want to add another region or cloud provider to this Deploy Config? y
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Credentials Name › mystartup-gcp
? GCP Project ID › my-gcp-project
? GCP Client Email › *******-compute@developer.gserviceaccount.com
? GCP Private Key › -----BEGIN PRIVATE KEY-----\*****\n-----END PRIVATE KEY-----\n
Successfully created "mystartup-gcp" credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › us-west1-c
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › e2-medium
? Do you want to add another region or cloud provider to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "production" Deploy Config.
```