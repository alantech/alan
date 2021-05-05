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
? Region name › us-west-1
? Virtual Machine Type › t2.medium
? Do you want to add another region to this Deploy Config? y
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Region name › us-west-2
? Virtual Machine Type › t2.medium
Successfully created "production" Credentials
```

## Multiple clouds

Generate a Deploy Config across AWS region `us-west-1` and GCP region `us-west1-c`.

```bash
$ anycloud config new
? Name for new Deploy Config (staging) › production
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Region name › us-west-1
? Virtual Machine Type › t2.medium
Do you want to add another region to this Deploy Config? y
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Credentials Name › mystartup-gcp
? GCP Project ID › my-gcp-project
? GCP Client Email › *******-compute@developer.gserviceaccount.com
? GCP Private Key › -----BEGIN PRIVATE KEY-----\*****\n-----END PRIVATE KEY-----\n
Successfully created "mystartup-gcp" credentials
? Region name › us-west1-c
? Virtual Machine Type › e2-medium
Successfully created "production" Credentials
```