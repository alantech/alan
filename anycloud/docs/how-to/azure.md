## Enable programmatic Azure access for AnyCloud

1) Create an Azure Active Directory application and its Service Principal as described [here](https://docs.microsoft.com/en-us/azure/active-directory/develop/howto-create-service-principal-portal). After following the previous instructions you can copy your `Application (client) ID` and your `Directory (tenant) ID`.

2) Create a secret for the service principal created above as described [here](https://docs.microsoft.com/en-us/azure/active-directory/develop/howto-create-service-principal-portal#option-2-create-a-new-application-secret). Copy the secret `Value` immediately since you will not be able to retrieve it again later.

3) For the subscription ID you can go to your [subscriptions page](https://portal.azure.com/#blade/Microsoft_Azure_Billing/SubscriptionsBlade) in Azure portal and get the ID.

4) To be able to use Anycloud with Azure you will need to manage your subscription resource provider registration as described [here](https://docs.microsoft.com/en-us/azure/azure-resource-manager/templates/error-register-resource-provider#solution-3---azure-portal). You will need to register: `Microsoft.Compute`, `Microsoft.Network`, `Microsoft.Storage` and `Microsoft.Security`.

5) Add a new `Credentials` by taking the values from the previous steps.

{% hint style="info" %}
You will need to pick a name, or alias, for the `Credentials`. The default value will be `azure`. In this example, we will call it `mystartup-azure`.
{% endhint %}

```bash
$ anycloud credentials new
? Pick cloud provider for the new Credentials ›
  AWS
  GCP
❯ Azure
? Credentials Name › mystartup-azure
? Azure Application ID › ********-****-****-****-************
? Azure Directory ID › ********-****-****-****-************
? Azure Subscription ID › ********-****-****-****-************
? Azure Secret › **********************************
Successfully created "mystartup-azure" Credentials
```

## Configure your project

Define a new `Deploy Config` in the `anycloud.json` project you want to deploy to Azure using the AnyCloud CLI

{% hint style="info" %}
You will need to pick a name, or alias, for the `Deploy Config`. The default value will be `staging`. You will also need to associate `Credentials` to this `Deploy Config`.
{% endhint %}

```bash
$ anycloud config new
Name for new Deploy Config: staging
? Pick Credentials to use ›
❯ mystartup-azure
  Create new Credentials
? Do you want to choose a specific region for this Deploy Config? › y
? Region name › eastus
? Do you want to select which virtual machine type to use for this Deploy Config? › y
? Virtual Machine Type › Standard_B1s
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "staging" Deploy Config.
```
