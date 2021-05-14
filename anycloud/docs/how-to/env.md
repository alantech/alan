To pass in enviroment variables when creating or upgrading an AnyCloud app define the optional `-e`, or `--env-file`, parameter that is a path to a `.env` file

```bash
$ anycloud new -e staging.env
```

The `.env` file has the following format

{% code title="staging.env" %}
```
PORT=8080
PROD=true
URL=staging.example.com
```
{% endcode %}