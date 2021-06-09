## Enable programmatic AWS access to VMs for AnyCloud

1) Create a new IAM user in your AWS account using their console/UI as described [here](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_users_create.html#id_users_create_console).

2) Create a new access key under that IAM user using their console/UI as described [here](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_access-keys.html#Using_CreateAccessKey).

3) Enable programmatic access for that IAM user, and attach the built-in [`AmazonEC2FullAccess`](https://console.aws.amazon.com/iam/home#/policies/arn%3Aaws%3Aiam%3A%3Aaws%3Apolicy%2FAmazonEC2FullAccess)policy to it as described [here](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_manage-attach-detach.html#add-policies-console).

4) Take the `accessKeyId` and `secretAccessKey` from step 2 and create AWS `Credentials` stored locally at `~/.anycloud/credentials.json` only.

{% hint style="info" %}
You will need to pick a name or alias for the `Credentials`. The default value will be `aws`. In this example, we will call it `mystartup-aws`.
{% endhint %}

```bash
$ anycloud credentials new
? Pick cloud provider for the new credentials ›
❯ AWS
  GCP
  Azure
Name for new Credentials: mystartup-aws
AWS Access Key ID: ******************
AWS Secret Access Key: ******************
Successfully created "mystartup-aws" Credentials
```

## Configure your project

Define a new `Deploy Config` in the `anycloud.json` project you want to deploy to AWS using the AnyCloud CLI.

{% hint style="info" %}
You will need to pick a name, or alias, for the `Deploy Config`. The default value will be `staging`. You will also need to associate `Credentials` to this `Deploy Config`.
{% endhint %}

```bash
$ anycloud config new
? Name for new Deploy Config › staging
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › us-east-1
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › t2.micro
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "staging" Deploy Config.
```