## .git

The AnyCloud CLI expects your project to be version controlled with `git`. However, it is not required for your repository to be hosted in a remote `git` server like GitHub or GitLab.

## Dockerfile

Deploying an AnyCloud `App` requires a [`Dockerfile`](https://docs.docker.com/engine/reference/builder/) file located within the top level folder of your project next to your `anycloud.json`. AnyCloud will expect the docker container described by the `Dockerfile` to have a HTTP server listening on port `8088` like the one [here](../tutorial.md#configure-your-project).

## anycloud.json

AnyCloud `Deploy Configs` are stored in an `anycloud.json` within the project directory, next to your `Dockerfile`, and configured using the CLI subcommands under `anycloud config`. Each `Deploy Config` consists of a name, or alias, to refer to it when creating or updating `Apps`. A `Deploy Config` consists of one or multiple of the following configurations: region, virtual machine type and an existing [`Credentials`](credentials.md) name or alias.

```bash
$ anycloud config add
Name for new Deploy Config: staging
Pick Credentials to use:
> mystartup-aws
? Do you want to choose a specific region for this Deploy Config? › y
Region name: us-west-1
? Do you want to select which virtual machine type to use for this Deploy Config? › y
Virtual Machine Type: t2.medium
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "staging" Deploy Config.

$ anycloud config add
Name for new Deploy Config: production
Pick Credentials to use:
> mystartup-aws
? Do you want to choose a specific region for this Deploy Config? › y
? Region name: us-west-1
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type: t2.xlarge
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 2
? Would you like to define a maximum number of VMs? › n
Successfully created "production" Deploy Config.

$ anycloud config list

Deployment configurations:

┌────────────┬─────────────────┬───────────┬───────────┐
│ Name       │ Credential Name │ Region    │ VM Type   │
├────────────┼─────────────────┼───────────┼───────────┤
│ production │ mystartup-aws   │ us-west-1 │ t2.xlarge │
│ staging    │ mystartup-aws   │ us-west-1 │ t2.medium │
└────────────┴─────────────────┴───────────┴───────────┘
```

The resulting `anycloud.json` contains two `Deploy Config`s called `staging` and `production` and looks like this:

```javascript
{
  "staging": [{
    "cloudProvider": "AWS",
    "region": "us-west-1",
    "vmType": "t2.medium",
    "credentialsName": "mystartup-aws",
    "minReplicas": 1
  }],
  "production": [{
    "cloudProvider": "AWS",
    "region": "us-west-1",
    "vmType": "t2.xlarge",
    "credentialsName": "mystartup-aws",
    "minReplicas": 2
  }]
}
```

Each cloud provider will have a different possible values for region and virtual machine type.

* **AWS**: List of available [regions](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html#concepts-available-regions) and [virtual machine types](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-types.html#AvailableInstanceTypes)
* **GCP**: List of available [regions](https://cloud.google.com/compute/docs/regions-zones#available) and [virtual machines types](https://cloud.google.com/compute/docs/machine-types)
* **Azure**: List of available [regions](https://azure.microsoft.com/en-us/global-infrastructure/geographies/#geographies) and [virtual machines types](https://docs.microsoft.com/en-us/azure/virtual-machines/sizes)

Note: AnyCloud does not currently support any ARM based VMs.