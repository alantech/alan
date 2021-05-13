# Persist data to PostgreSQL in AWS

The guide covers how to deploy a Node.js REST HTTP API to AWS using AnyCloud. The HTTP server persists data to a PostgreSQL instance in AWS RDS. We will use the ORM [Prisma](https://www.prisma.io) and start from their [rest-express](https://github.com/prisma/prisma-examples/tree/latest/typescript/rest-express) example. This is one of Prisma's ready-to-run example projects demonstrating a backend-only HTTP server written using Typescript, Node.js and [Express](https://expressjs.com). The REST endpoints use Prisma to persist and retrieve data from a PostgreSQL instance.

All the code from this guide can be found in this [template repository](https://github.com/alantech/anycloud-node-aws-psql) which you can use to [create a new repository](https://docs.github.com/en/github/creating-cloning-and-archiving-repositories/creating-a-repository-from-a-template) as a starting point for your project.

## Setup your project locally

1) First download Prisma's example and install dependencies

```
curl https://codeload.github.com/prisma/prisma-examples/tar.gz/latest | tar -xz --strip=2 prisma-examples-latest/typescript/rest-express
mv rest-express anycloud-node-aws-psql
cd anycloud-node-aws-psql
npm install
```

2) Install and start a local PostgreSQL instance by following Prisma's [guide](https://www.prisma.io/dataguide/postgresql/setting-up-a-local-postgresql-database) on how to do it.

3) Now point your project's Node.js REST Prisma ORM to the local PostgreSQL instance you just created by setting the [datasource](https://www.prisma.io/docs/concepts/database-connectors/postgresql/) in the Prisma schema file at `anycloud-node-aws-psql/prisma/schema.prisma` to the environment variable `DATABASE_URL`.

```
datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}
```

Then set the environment variable `DATABASE_URL` in the `.env` file by replacing `USER`, `PASSWORD`, `HOST`, `PORT` and `DATABASE` with the appropriate values

```
DATABASE_URL=postgresql://<USER>:<PASSWORD>@<HOST>:<PORT>/<DATABASE>
```

The value for your local PostgreSQL should be something like

```
DATABASE_URL=postgresql://postgres@localhost:5432/postgres
```

4) Create the initial database migration which creates the `User` and `Post` tables that are defined in `anycloud-node-aws-psql/prisma/schema.prisma` and a migrations folder with a timestamped migration named `init`

```
npx prisma migrate dev --name init
```

5) Add `scripts` to the `package.json` file

```
...

"scripts": {
  "build": "tsc",
  "clean": "rm -rf dist",
  "start": "node ./dist/src/index.js",
}

...
```

6) Define the `Dockerfile`

```bash
FROM node:lts

COPY . .

RUN npm install
RUN npm run build
CMD npm run start
```

7) Initialize a `git` repository and commit your changes

```bash
git init
git add .
git commit -m "Initial commit"
```

8) Change HTTP server port in `src/index.ts` from `3000` to `8088` which is what what AnyCloud expects.

```
...

const server = app.listen(8088, () =>
  console.log(`
🚀 Server ready at: http://localhost:8088

...
```

9) Make sure everything works locally

```
$ npm run start

> rest-express@1.0.0 start
> npx prisma migrate deploy && node ./dist/src/index.js

Environment variables loaded from .env
Prisma schema loaded from prisma/schema.prisma
Datasource "db": PostgreSQL database "postgres", schema "public" at "localhost:5432"

1 migration found in prisma/migrations

No pending migrations to apply.

🚀 Server ready at: http://localhost:8088
⭐️ See sample requests: http://pris.ly/e/ts/rest-express#3-using-the-rest-api
```

## Create a PostgreSQL instance on AWS RDS

Follow this [AWS guide](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/CHAP_GettingStarted.CreatingConnecting.PostgreSQL.html#CHAP_GettingStarted.Creating.PostgreSQL) to create a PostgreSQL on AWS RDS. Make sure to pick the `Easy create` method.

## Enable programmatic AWS access to VMs for AnyCloud

1) Create a new IAM user in your AWS account using their console/UI as described [here](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_users_create.html#id_users_create_console).

2) Create a new access key under that IAM user using their console/UI as described [here](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_access-keys.html#Using_CreateAccessKey).

3) Enable programmatic access for that IAM user, and attach the built-in [`AmazonEC2FullAccess`](https://console.aws.amazon.com/iam/home#/policies/arn%3Aaws%3Aiam%3A%3Aaws%3Apolicy%2FAmazonEC2FullAccess) policy to it as described [here](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_manage-attach-detach.html#add-policies-console).

4) Take the `accessKeyId` and `secretAccessKey` from step 2 and create AWS `Credentials` stored locally at `~/.anycloud/credentials.json` only. You will need to pick a name or alias for the `Credentials`. The default value will be `aws`. In this example, we will call it `mystartup-aws`.

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

## Deploy the REST API using AnyCloud and link it to the PostgreSQL instance

1) Use the AnyCloud CLI to create an `anycloud.json` file in the project directory and define a `Deploy Config`. You will need to pick a name or alias for the `Deploy Config`. The default value will be `staging`. You will also need to associate `Credentials` to this `Deploy Config`.

```bash
$ anycloud config new
? Name for new Deploy Config › staging
? Pick Credentials to use ›
❯ mystartup-aws
  Create new Credentials
? Region name › us-east-1
? Virtual Machine Type › m5.large
? Do you want to add another region to this Deploy Config? › n
? Minimum number of VMs per region or cloud › 1
? Would you like to define a maximum number of VMs? › n
Successfully created "staging" Deploy Config
```

2) Deploy the Node.js server to your AWS account using the AnyCloud CLI

```bash
$ anycloud new
? Pick Deploy Config for App ›
❯ staging
? Optional App name ›
▇ Creating new App
```

3) Check the id, or name, of your App using `anycloud list`

```bash
$ anycloud list
Apps deployed:

┌────────────────┬───────────────────────────────────────┬───────────────┬──────┬────────┐
│ App ID         │ Url                                   │ Deploy Config │ Size │ Status │
├────────────────┼───────────────────────────────────────┼───────────────┼──────┼────────┤
│ crimson-tick-5 │ https://crimson-tick-5.alandeploy.com │ staging       │ 1    │ up     │
└────────────────┴───────────────────────────────────────┴───────────────┴──────┴────────┘

Deploy Configs used:

┌───────────────┬───────────┬───────────┐
│ Deploy Config │ Region    │ VM Type   │
├───────────────┼───────────┼───────────┤
│ staging       │ us-east-1 │ m5.large  │
└───────────────┴───────────┴───────────┘
```

4) Now go to [AWS RDS](https://console.aws.amazon.com/rds/home) website and select the PostgreSQL instance you just created in order to include the [Security Group](https://console.aws.amazon.com/ec2/v2/home?#SecurityGroups) for the newly created AnyCloud app. First click `Modify`

<img src="../assets/modify-rds.png" />

Now scroll down to the `Connectivity` section and add an additional Security Group from the dropdown menu. The Security Group will have the format `anycloud-${app-id}-${uuid}`, so something like `anycloud-crimson-tick-5-234rfwerg`.

<img src="../assets/add-connectivity.png" />

5) Now point your project's Node.js REST Prisma ORM to the hosted PostgreSQL instance by changing the environment variable `DATABASE_URL` in the `.env` file to something like

```
DATABASE_URL=postgresql://postgres:XIJw6Gf4grPucMVcSuMe@database-2.cwryagwpxlyd.us-west-1.rds.amazonaws.com:5432/postgres
```

Again replacing `USER`, `PASSWORD`, `HOST`, `PORT` and `DATABASE` with the appropiate values

```
DATABASE_URL=postgresql://<USER>:<PASSWORD>@<HOST>:<PORT>/<DATABASE>
```

6) Let's make sure we run migrations on the hosted PostgreSQL when starting, or upgrading, our Node.js app by modifying the previously defined `start` script in our `package.json` file.

```
...

"scripts": {
  "start": "npx prisma migrate deploy && node ./dist/src/index.js",
  ...
},

...
```

7) Finally, commit your changes and upgrade the new app to have the changes take effect.

```
git add .
git commit -m "anycloud setup"
anycloud upgrade -e .env
```
