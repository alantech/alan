# Pass environment variables

To pass in enviroment variables when creating or upgrading an AnyCloud app define the optional `-e`, or `--env-file`, parameter that is a path to a [`.env` file](https://docs.docker.com/compose/env-file/):

```bash
$ anycloud new staging -e staging.env
```

