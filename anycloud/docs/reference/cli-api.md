# CLI API

## help

```
$ anycloud help
Elastically scale webservers in any cloud provider

USAGE:
    anycloud <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    config         Manage Deploy Configs used by Apps from the anycloud.json in the current directory
    credentials    Manage all Credentials used by Deploy Configs from the credentials file at
                   ~/.anycloud/credentials.json
    help           Prints this message or the help of the given subcommand(s)
    list           Displays all the Apps deployed with the Deploy Configs from anycloud.json
    new            Deploys your repository to a new App with a Deploy Config from anycloud.json
    terminate      Terminate an App hosted in one of the Deploy Configs from anycloud.json
    upgrade        Deploys your repository to an existing App hosted in one of the Deploy Configs from anycloud.json
```

## new

```
$ anycloud help new
Deploys your repository to a new App with a Deploy Config from anycloud.json

USAGE:
    anycloud new [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --env-file <ENV_FILE>    Specifies an optional environment file
```

## upgrade

```
$ anycloud help upgrade
Deploys your repository to an existing App hosted in one of the Deploy Configs from anycloud.json

USAGE:
    anycloud upgrade [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --env-file <ENV_FILE>    Specifies an optional environment file relative path
```

## list

```
$ anycloud help list
Displays all the Apps deployed with the Deploy Configs from anycloud.json

USAGE:
    anycloud list

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

## terminate

```
$ anycloud help terminate
Terminate an App hosted in one of the Deploy Configs from anycloud.json

USAGE:
    anycloud terminate

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

## credentials

```
$ anycloud help credentials
Manage all Credentials used by Deploy Configs from the credentials file at ~/.anycloud/credentials.json

USAGE:
    anycloud credentials <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    new       Add a new Credentials
    edit      Edit an existing Credentials
    help      Prints this message or the help of the given subcommand(s)
    list      List all the available Credentials
    remove    Remove an existing Credentials
```

## config

```
$ anycloud help config
Manage Deploy Configs used by Apps from the anycloud.json in the current directory

USAGE:
    anycloud config <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    new       Add a new Deploy Config to the anycloud.json in the current directory and creates the file if it
              doesn't exist.
    edit      Edit an existing Deploy Config from the anycloud.json in the current directory
    help      Prints this message or the help of the given subcommand(s)
    list      List all the Deploy Configs from the anycloud.json in the current directory
    remove    Remove an existing Deploy Config from the anycloud.json in the current directory
```