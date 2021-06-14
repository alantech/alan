<img src="../assets/azure-node.jpg" />

In this tutorial we will deploy the [sample express Node.js HTTP server](https://expressjs.com/en/starter/hello-world.html) in your own Azure account with [AnyCloud](https://anycloudapp.com).

{% hint style="info" %}
All the code can be found in this [template repository](https://github.com/alantech/hello-anycloud) which you can use to [create a new repository](https://docs.github.com/en/github/creating-cloning-and-archiving-repositories/creating-a-repository-from-a-template) for your AnyCloud project.
{% endhint %}

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

1) Initialize a `git` repository

```bash
git init
git add -A
git commit -m "Initial commit"
```

2) Initialize your `package.json` and install `express`

```bash
npm init
npm install express --save
```

3) Define an HTTP server listening on port `8088` in an `index.js` file

```javascript
const express = require('express')
const app = express()
const port = 8088

app.get('/', (req, res) => {
  res.send('Hello World!')
})

app.listen(port, () => {
  console.log(`Example app listening at http://localhost:${port}`)
})
```

4) Define the `Dockerfile`

```bash
FROM node:lts

COPY . .

RUN npm install
CMD node index.js
```

5) Test the `Dockerfile` locally by installing [Docker Desktop](https://www.docker.com/products/docker-desktop), building the Docker image and then running the server within the container

```bash
$ docker build -t anycloud/app .
$ docker run -p 8088:8088 -d anycloud/app:latest
$ curl localhost:8088
```

Which should return `Hello World!`

6) Use the AnyCloud CLI to create an `anycloud.json` file in the project directory and define a `Deploy Config`.

{% hint style="info" %}
You will need to pick a name, or alias, for the `Deploy Config`. The default value will be `staging`. You will also need to associate `Credentials` to this `Deploy Config`.
{% endhint %}

```bash
$ anycloud config new
Name for new Deploy Config: staging-azure
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
Successfully created "staging-azure" Deploy Config.
```

7) Make sure all of the changes in the git repo are committed or they won't be deployed.

## Deploy an App

1) Make sure you [installed the AnyCloud CLI](about.md#cli-installation). Now deploy your Node.js server to your Azure account using the AnyCloud CLI.

```bash
$ anycloud new
? Pick Deploy Config for App ›
❯ staging-azure
? Optional App name ›
▇ Creating new App
```
{% hint style="info" %}
It might take a few minutes for your App to start while the virtual machine is provisioned and upgraded.
{% endhint %}

2) Check the status of your App

```bash
$ anycloud list
Apps deployed:

┌────────────────┬────────────────────────────────────────┬──────────────────────┬──────┬────────┐
│ App ID         │ Url                                    │ Deploy Config        │ Size │ Status │
├────────────────┼────────────────────────────────────────┼──────────────────────┼──────┼────────┤
│ crimson-tick-5 │ https://crimson-tick-5.anycloudapp.com │ staging-azure        │ 1    │ up     │
└────────────────┴────────────────────────────────────────┴──────────────────────┴──────┴────────┘

Deploy Configs used:

┌─────────────────────┬───────────┬──────────────┐
│ Deploy Config       │ Region    │ VM Type      │
├─────────────────────┼───────────┼──────────────┤
│ staging-azure       │ eastus    │ Standard_B1s │
└─────────────────────┴───────────┴──────────────┘

```

3) The `size` of your App represents the number of virtual machines used to back your App. Apps scale elastically based on request load automatically. Now `curl` your AnyCloud App!

```bash
$ curl https://crimson-tick-5.anycloudapp.com
```

Which should return `Hello World!`

4) Terminate your AnyCloud App when you no longer need it

```bash
anycloud terminate
? Pick App to terminate ›
❯ crimson-tick-5
```
