# Deploy to GCP

## Enable programmatic GCP access for AnyCloud

1) Create a service account for your GCP project as described [here](https://cloud.google.com/iam/docs/creating-managing-service-accounts#iam-service-accounts-create-console) with the [`Compute Engine Admin role`](https://cloud.google.com/compute/docs/access/iam#compute.admin).

2) Create a service account key for your newly service account as described [here](https://cloud.google.com/iam/docs/creating-managing-service-account-keys) and export it as a JSON file.

3) Take a look at the exported JSON file. Add a new `Credentials` by taking the `privateKey`, `clientEmail` and `projectId` from step 2. You will need to pick a name or alias for the `Credentials`. The initial value will be `gcp`. In this example, we will call it `mystartup-gcp`.

```bash
$ anycloud credentials new
? Pick cloud provider for the new credentials ›
  AWS
❯ GCP
  Azure
? Credentials Name › mystartup-gcp
? GCP Project ID › my-gcp-project
? GCP Client Email › *******-compute@developer.gserviceaccount.com
? GCP Private Key › -----BEGIN PRIVATE KEY-----\*****\n-----END PRIVATE KEY-----\n
Successfully created "mystartup-gcp" credentials
```

## Configure your project

Define a new `Deploy Config` in the `anycloud.json` project you want to deploy to GCP using the AnyCloud CLI:

```bash
$ anycloud config add
Name for new Deploy Config: staging
? Pick Credentials to use ›
❯ mystartup-gcp
  Create new Credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › us-east4-a
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › e2-standard-2
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "staging" Deploy Config.
```


