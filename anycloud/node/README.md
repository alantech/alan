# AnyCloud

AnyCloud is a Lambda alternative for Node.js that works with multiple cloud providers. We’re building the features of Lambda while fixing the problems we have run into while using it. Namely:

- Vendor lock-in
- Cold starts
- Limited runtime (10-15m)
- Stateless
- Cumbersome to run locally
- Unintuitive to version and remove inactive functions
- HTTPS support not included

## Documentation

Visit our [docs](https://alantech.gitbook.io/anycloud)

## Status
- [ ] Public Alpha: Anyone can sign up, but go easy on us
- [ ] Public Beta: Stable enough for most non-enterprise use-cases
- [ ] Public: Production-ready

## Installation

Simply `npm i -g anycloud` to get access to the `anycloud` cli application.

## Usage

TODO

## How it works

AnyCloud is a serverless framework built on the Alan and Rust programming languages that:

- [x] Automatically scales your http server across many machines based on request load and system stats
- [x] Works on multiple cloud providers so you are not locked into AWS
- [x] Runs locally as-is without special configuration of your local dev environment
- [ ] In-memory distributed datastore

Our aim is to give developers a much better experience when using AnyClloud than Lambda.

**Public Cloud Providers**

AnyCloud is hosted directly in your account with the preferred cloud provider. Cloud providers currently supported:
- [x] AWS
- [x] GCP
- [ ] Azure (coming soon)

You start using Anycloud without signing up.